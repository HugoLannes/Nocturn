import { useCallback, useEffect, useRef, useState } from "react";
import { relaunch } from "@tauri-apps/plugin-process";
import { check, type DownloadEvent, type Update } from "@tauri-apps/plugin-updater";

type InstallState = "idle" | "downloading" | "installing" | "relaunching" | "error";

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

export function useUpdater() {
  const [state, setState] = useState<UseUpdaterState>(INITIAL_STATE);
  const updateRef = useRef<Update | null>(null);

  const checkForUpdate = useCallback(async () => {
    if (import.meta.env.DEV) {
      setState(INITIAL_STATE);
      return null;
    }

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
      setState((current) => ({
        ...current,
        installState: "error",
        downloadProgress: null,
        errorMessage: "Unable to install the update. Try again.",
      }));
    }
  }, []);

  return {
    ...state,
    checkForUpdate,
    installUpdate,
  };
}
