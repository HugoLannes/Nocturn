import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Display, DisplayUpdatePayload } from "../types";

type UseDisplaysState = {
  displays: Display[];
  isLoading: boolean;
  activeDisplayCount: number;
  blackoutCount: number;
  allowCursorExitActiveDisplays: boolean;
};

const INITIAL_STATE: UseDisplaysState = {
  displays: [],
  isLoading: true,
  activeDisplayCount: 0,
  blackoutCount: 0,
  allowCursorExitActiveDisplays: true,
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

export function useDisplays() {
  const [state, setState] = useState<UseDisplaysState>(INITIAL_STATE);
  const [isMutating, setIsMutating] = useState(false);
  const [pendingDisplayId, setPendingDisplayId] = useState<string | null>(null);

  const loadDisplays = useCallback(async () => {
    try {
      const payload = await withTimeout(invoke<DisplayUpdatePayload>("get_displays"), "Loading displays");
      setState((current) => ({
        ...current,
        displays: payload.displays,
        activeDisplayCount: payload.activeDisplayCount,
        blackoutCount: payload.blackoutCount,
        allowCursorExitActiveDisplays: payload.allowCursorExitActiveDisplays,
        isLoading: false,
      }));
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
      setState((current) => ({
        ...current,
        displays: event.payload.displays,
        activeDisplayCount: event.payload.activeDisplayCount,
        blackoutCount: event.payload.blackoutCount,
        allowCursorExitActiveDisplays: event.payload.allowCursorExitActiveDisplays,
        isLoading: false,
      }));
    }).then((cleanup) => {
      unlisten = cleanup;
    });

    return () => {
      void unlisten?.();
    };
  }, [loadDisplays]);

  const toggleDisplay = useCallback(async (displayId: string) => {
    setIsMutating(true);
    setPendingDisplayId(displayId);

    try {
      await withTimeout(invoke<string>("toggle_display", { id: displayId }), "Toggling display");
    } catch (error) {
      console.error(`Failed to toggle display ${displayId}:`, error);
      void loadDisplays();
    } finally {
      setIsMutating(false);
      setPendingDisplayId(null);
    }
  }, [loadDisplays]);

  const wakeAll = useCallback(async () => {
    setIsMutating(true);
    setPendingDisplayId(null);

    try {
      await withTimeout(invoke("unblank_all"), "Waking displays");
    } catch (error) {
      console.error("Failed to wake displays:", error);
      void loadDisplays();
    } finally {
      setIsMutating(false);
    }
  }, [loadDisplays]);

  const focusPrimary = useCallback(async () => {
    setIsMutating(true);
    setPendingDisplayId(null);

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

      setState((current) => ({
        ...current,
        displays: payload.displays,
        activeDisplayCount: payload.activeDisplayCount,
        blackoutCount: payload.blackoutCount,
        allowCursorExitActiveDisplays: payload.allowCursorExitActiveDisplays,
      }));
    } catch (error) {
      console.error("Failed to update cursor setting:", error);
      void loadDisplays();
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
    pendingDisplayId,
    loadDisplays,
    toggleDisplay,
    wakeAll,
    focusPrimary,
    setAllowCursorExitActiveDisplays,
    lastActiveDisplayId,
  };
}
