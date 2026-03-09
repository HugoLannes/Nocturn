import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Display, DisplayUpdatePayload, ShortcutSettings, ShortcutSettingsInput } from "../types";

type UseDisplaysState = {
  displays: Display[];
  isLoading: boolean;
  activeDisplayCount: number;
  blackoutCount: number;
  allowCursorExitActiveDisplays: boolean;
  showOverlayHiddenApps: boolean;
  shortcutSettings: ShortcutSettings;
};

const EMPTY_SHORTCUT_SETTINGS: ShortcutSettings = {
  focusModeHotkey: null,
  displayBindings: [],
};

const INITIAL_STATE: UseDisplaysState = {
  displays: [],
  isLoading: true,
  activeDisplayCount: 0,
  blackoutCount: 0,
  allowCursorExitActiveDisplays: true,
  showOverlayHiddenApps: true,
  shortcutSettings: EMPTY_SHORTCUT_SETTINGS,
};

const COMMAND_TIMEOUT_MS = 5000;

function withTimeout<T>(promise: Promise<T>, label: string, timeoutMs = COMMAND_TIMEOUT_MS): Promise<T> {
  return new Promise((resolve, reject) => {
    const timer = window.setTimeout(() => {
      reject(new Error(`${label} timed out after ${timeoutMs}ms.`));
    }, timeoutMs);

    void promise.then(
      (value) => {
        window.clearTimeout(timer);
        resolve(value);
      },
      (error: unknown) => {
        window.clearTimeout(timer);
        reject(error);
      },
    );
  });
}

function applyPayload(current: UseDisplaysState, payload: DisplayUpdatePayload): UseDisplaysState {
  return {
    ...current,
    displays: payload.displays,
    activeDisplayCount: payload.activeDisplayCount,
    blackoutCount: payload.blackoutCount,
    allowCursorExitActiveDisplays: payload.allowCursorExitActiveDisplays,
    showOverlayHiddenApps: payload.showOverlayHiddenApps,
    shortcutSettings: payload.shortcutSettings,
    isLoading: false,
  };
}

function getErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }

  if (typeof error === "string") {
    return error;
  }

  return "Unknown error.";
}

export function useDisplays() {
  const [state, setState] = useState<UseDisplaysState>(INITIAL_STATE);
  const [isMutating, setIsMutating] = useState(false);
  const [pendingDisplayIds, setPendingDisplayIds] = useState<ReadonlySet<string>>(new Set<string>());

  const loadDisplays = useCallback(async () => {
    try {
      const payload = await withTimeout(invoke<DisplayUpdatePayload>("get_displays"), "Loading displays");
      setState((current) => applyPayload(current, payload));
    } catch (error) {
      console.error("Failed to load displays:", error);
      setState((current) => ({
        ...current,
        isLoading: false,
      }));
    }
  }, []);

  useEffect(() => {
    let unlisten: UnlistenFn | undefined;

    void loadDisplays();

    void listen<DisplayUpdatePayload>("displays-update", (event) => {
      setState((current) => applyPayload(current, event.payload));
    }).then((cleanup) => {
      unlisten = cleanup;
    });

    return () => {
      void unlisten?.();
    };
  }, [loadDisplays]);

  const toggleDisplay = useCallback(async (displayId: string) => {
    setPendingDisplayIds((prev) => new Set([...prev, displayId]));

    // Optimistic update: flip the display state immediately so the UI responds
    // without waiting for the backend round-trip.
    setState((current) => {
      const display = current.displays.find((d) => d.id === displayId);
      if (!display) return current;
      const willBeBlackedOut = !display.isBlackedOut;
      return {
        ...current,
        displays: current.displays.map((d) =>
          d.id === displayId ? { ...d, isBlackedOut: willBeBlackedOut } : d,
        ),
        activeDisplayCount: willBeBlackedOut
          ? current.activeDisplayCount - 1
          : current.activeDisplayCount + 1,
        blackoutCount: willBeBlackedOut
          ? current.blackoutCount + 1
          : current.blackoutCount - 1,
      };
    });

    try {
      await withTimeout(invoke<string>("toggle_display", { id: displayId }), "Toggling display");
    } catch (error) {
      console.error(`Failed to toggle display ${displayId}:`, error);
      // Revert the optimistic update on failure.
      setState((current) => {
        const display = current.displays.find((d) => d.id === displayId);
        if (!display) return current;
        const revertToBlackedOut = !display.isBlackedOut;
        return {
          ...current,
          displays: current.displays.map((d) =>
            d.id === displayId ? { ...d, isBlackedOut: revertToBlackedOut } : d,
          ),
          activeDisplayCount: revertToBlackedOut
            ? current.activeDisplayCount - 1
            : current.activeDisplayCount + 1,
          blackoutCount: revertToBlackedOut
            ? current.blackoutCount + 1
            : current.blackoutCount - 1,
        };
      });
      void loadDisplays();
    } finally {
      setPendingDisplayIds((prev) => {
        const next = new Set(prev);
        next.delete(displayId);
        return next;
      });
    }
  }, [loadDisplays]);

  const restoreAllDisplays = useCallback(async () => {
    setIsMutating(true);

    try {
      await withTimeout(invoke("unblank_all"), "Restoring displays");
    } catch (error) {
      console.error("Failed to restore displays:", error);
      void loadDisplays();
    } finally {
      setIsMutating(false);
    }
  }, [loadDisplays]);

  const focusPrimary = useCallback(async () => {
    setIsMutating(true);

    try {
      await withTimeout(invoke("focus_primary"), "Enabling focus mode");
    } catch (error) {
      console.error("Failed to enable focus mode:", error);
      void loadDisplays();
    } finally {
      setIsMutating(false);
    }
  }, [loadDisplays]);

  const setAllowCursorExitActiveDisplays = useCallback(async (allowed: boolean) => {
    setIsMutating(true);

    try {
      const payload = await withTimeout(
        invoke<DisplayUpdatePayload>("set_allow_cursor_exit_active_displays", { allowed }),
        "Updating cursor setting",
      );

      setState((current) => applyPayload(current, payload));
    } catch (error) {
      console.error("Failed to update cursor setting:", error);
      void loadDisplays();
    } finally {
      setIsMutating(false);
    }
  }, [loadDisplays]);

  const setShowOverlayHiddenApps = useCallback(async (enabled: boolean) => {
    setIsMutating(true);

    try {
      const payload = await withTimeout(
        invoke<DisplayUpdatePayload>("set_show_overlay_hidden_apps", { enabled }),
        "Updating overlay app labels",
      );

      setState((current) => applyPayload(current, payload));
    } catch (error) {
      console.error("Failed to update overlay app labels:", error);
      void loadDisplays();
    } finally {
      setIsMutating(false);
    }
  }, [loadDisplays]);

  const setShortcutSettings = useCallback(async (hotkeys: ShortcutSettingsInput) => {
    setIsMutating(true);

    try {
      const payload = await withTimeout(
        invoke<DisplayUpdatePayload>("set_shortcut_settings", { hotkeys }),
        "Updating shortcut settings",
      );

      setState((current) => applyPayload(current, payload));
      return null;
    } catch (error) {
      console.error("Failed to update shortcut settings:", error);
      void loadDisplays();
      return getErrorMessage(error);
    } finally {
      setIsMutating(false);
    }
  }, [loadDisplays]);

  const lastActiveDisplayId = useMemo(() => {
    if (state.activeDisplayCount !== 1) {
      return null;
    }

    const activeDisplay = state.displays.find((display) => !display.isBlackedOut);
    return activeDisplay?.id ?? null;
  }, [state.activeDisplayCount, state.displays]);

  return {
    ...state,
    isMutating,
    pendingDisplayIds,
    loadDisplays,
    toggleDisplay,
    restoreAllDisplays,
    focusPrimary,
    setAllowCursorExitActiveDisplays,
    setShowOverlayHiddenApps,
    setShortcutSettings,
    lastActiveDisplayId,
  };
}
