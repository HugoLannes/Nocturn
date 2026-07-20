import { useCallback, useEffect, useRef, useState } from "react";
import { relaunch } from "@tauri-apps/plugin-process";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { check, type DownloadEvent, type Update } from "@tauri-apps/plugin-updater";

type InstallState = "idle" | "downloading" | "installing" | "relaunching" | "error";

// Nocturn is tray-resident and its webview is only hidden/shown, never
// remounted, so the mount-time check below is the only one that would ever
// run without these. RECHECK_INTERVAL_MS is the background safety net for a
// long-running tray session; FOCUS_RECHECK_MIN_INTERVAL_MS throttles the
// extra re-check fired when the panel regains focus (tray click / "Show
// Panel") so refocusing repeatedly doesn't hammer the update endpoint.
const RECHECK_INTERVAL_MS = 12 * 60 * 60 * 1000;
const FOCUS_RECHECK_MIN_INTERVAL_MS = 60 * 60 * 1000;

type UseUpdaterState = {
  isChecking: boolean;
  isUpdateAvailable: boolean;
  availableVersion: string | null;
  currentVersion: string | null;
  installState: InstallState;
  downloadProgress: number | null;
  errorMessage: string | null;
};

const INITIAL_STATE: UseUpdaterState = {
  isChecking: !import.meta.env.DEV,
  isUpdateAvailable: false,
  availableVersion: null,
  currentVersion: null,
  installState: "idle",
  downloadProgress: null,
  errorMessage: null,
};

async function disposeUpdate(update: Update | null) {
  if (!update) {
    return;
  }

  try {
    await update.close();
  } catch (error) {
    console.warn("Failed to dispose updater resource:", error);
  }
}

function getUpdaterErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }

  if (typeof error === "string") {
    return error;
  }

  return "Unknown error.";
}

export function useUpdater() {
  const [state, setState] = useState<UseUpdaterState>(INITIAL_STATE);
  const updateRef = useRef<Update | null>(null);
  const installStateRef = useRef<InstallState>(state.installState);
  const lastCheckedAtRef = useRef<number>(0);

  useEffect(() => {
    installStateRef.current = state.installState;
  }, [state.installState]);

  const checkForUpdate = useCallback(async () => {
    if (import.meta.env.DEV) {
      setState(INITIAL_STATE);
      return null;
    }

    // Never let a background/focus-triggered re-check dispose the Update
    // handle that installUpdate is actively downloading/installing through.
    if (
      installStateRef.current === "downloading" ||
      installStateRef.current === "installing" ||
      installStateRef.current === "relaunching"
    ) {
      return null;
    }

    lastCheckedAtRef.current = Date.now();

    setState((current) => ({
      ...current,
      isChecking: true,
      errorMessage: null,
    }));

    try {
      const nextUpdate = await check({ timeout: 15000 });

      if (updateRef.current && updateRef.current !== nextUpdate) {
        void disposeUpdate(updateRef.current);
      }

      updateRef.current = nextUpdate;

      setState((current) => ({
        ...current,
        isChecking: false,
        isUpdateAvailable: nextUpdate !== null,
        availableVersion: nextUpdate?.version ?? null,
        currentVersion: nextUpdate?.currentVersion ?? null,
        installState: nextUpdate ? current.installState : "idle",
        downloadProgress: nextUpdate ? current.downloadProgress : null,
        errorMessage: null,
      }));

      return nextUpdate;
    } catch (error) {
      console.error("Failed to check for updates:", error);
      void disposeUpdate(updateRef.current);
      updateRef.current = null;

      setState((current) => ({
        ...current,
        isChecking: false,
        isUpdateAvailable: false,
        availableVersion: null,
        currentVersion: null,
        installState: current.installState === "error" ? "error" : "idle",
        downloadProgress: null,
        errorMessage: "Unable to check for updates.",
      }));

      return null;
    }
  }, []);

  useEffect(() => {
    void checkForUpdate();

    return () => {
      void disposeUpdate(updateRef.current);
      updateRef.current = null;
    };
  }, [checkForUpdate]);

  // Background safety net: a tray-resident session can run for weeks without
  // the webview ever remounting, so re-check periodically instead of relying
  // solely on the mount-time check above.
  useEffect(() => {
    if (import.meta.env.DEV) {
      return;
    }

    const interval = window.setInterval(() => {
      void checkForUpdate();
    }, RECHECK_INTERVAL_MS);

    return () => window.clearInterval(interval);
  }, [checkForUpdate]);

  // Also re-check whenever the panel regains focus (tray click / "Show
  // Panel" both call set_focus() on the same never-remounted window), throttled
  // so repeated refocusing doesn't spam the update endpoint.
  useEffect(() => {
    if (import.meta.env.DEV) {
      return;
    }

    let isDisposed = false;
    let unlisten: (() => void) | undefined;

    void getCurrentWindow()
      .onFocusChanged(({ payload: isFocused }) => {
        if (!isFocused) {
          return;
        }

        if (Date.now() - lastCheckedAtRef.current < FOCUS_RECHECK_MIN_INTERVAL_MS) {
          return;
        }

        void checkForUpdate();
      })
      .then((cleanup) => {
        if (isDisposed) {
          cleanup();
          return;
        }

        unlisten = cleanup;
      });

    return () => {
      isDisposed = true;
      unlisten?.();
    };
  }, [checkForUpdate]);

  const installUpdate = useCallback(async () => {
    const update = updateRef.current;

    if (!update) {
      return;
    }

    let downloadedBytes = 0;
    let totalBytes = 0;

    setState((current) => ({
      ...current,
      installState: "downloading",
      downloadProgress: 0,
      errorMessage: null,
    }));

    try {
      await update.downloadAndInstall(
        (event: DownloadEvent) => {
          switch (event.event) {
            case "Started":
              downloadedBytes = 0;
              totalBytes = event.data.contentLength ?? 0;
              setState((current) => ({
                ...current,
                installState: "downloading",
                downloadProgress: totalBytes > 0 ? 0 : null,
              }));
              break;
            case "Progress":
              downloadedBytes += event.data.chunkLength;
              setState((current) => ({
                ...current,
                installState: "downloading",
                downloadProgress: totalBytes > 0 ? Math.min(100, Math.round((downloadedBytes / totalBytes) * 100)) : null,
              }));
              break;
            case "Finished":
              setState((current) => ({
                ...current,
                installState: "installing",
                downloadProgress: 100,
              }));
              break;
          }
        },
        { timeout: 300000 },
      );

      setState((current) => ({
        ...current,
        installState: "relaunching",
        downloadProgress: 100,
      }));

      await disposeUpdate(updateRef.current);
      updateRef.current = null;

      await relaunch();
    } catch (error) {
      console.error("Failed to install update:", error);
      const detail = getUpdaterErrorMessage(error);
      const isVerificationFailure = /signat|verif/i.test(detail);

      setState((current) => ({
        ...current,
        installState: "error",
        downloadProgress: null,
        errorMessage: isVerificationFailure
          ? `Update verification failed: ${detail}`
          : `Unable to install the update: ${detail}`,
      }));
    }
  }, []);

  return {
    ...state,
    checkForUpdate,
    installUpdate,
  };
}
