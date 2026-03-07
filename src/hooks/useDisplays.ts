import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Display, DisplayUpdatePayload } from "../types";

type UseDisplaysState = {
  displays: Display[];
  isLoading: boolean;
  error: string | null;
  feedback: string | null;
  activeDisplayCount: number;
  blackoutCount: number;
};

const INITIAL_STATE: UseDisplaysState = {
  displays: [],
  isLoading: true,
  error: null,
  feedback: null,
  activeDisplayCount: 0,
  blackoutCount: 0,
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

  const loadDisplays = useCallback(async (preserveError = false) => {
    try {
      const payload = await withTimeout(invoke<DisplayUpdatePayload>("get_displays"), "Loading displays");
      setState((current) => ({
        ...current,
        displays: payload.displays,
        activeDisplayCount: payload.activeDisplayCount,
        blackoutCount: payload.blackoutCount,
        error: preserveError ? current.error : null,
        isLoading: false,
      }));
    } catch (error) {
      setState((current) => ({
        ...current,
        error: error instanceof Error ? error.message : String(error),
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
        error: null,
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
    setState((current) => ({ ...current, feedback: null, error: null }));

    try {
      const feedback = await withTimeout(invoke<string>("toggle_display", { id: displayId }), "Toggling display");
      if (feedback) {
        setState((current) => ({ ...current, feedback }));
      }
    } catch (error) {
      setState((current) => ({
        ...current,
        error: error instanceof Error ? error.message : String(error),
      }));
      void loadDisplays(true);
    } finally {
      setIsMutating(false);
      setPendingDisplayId(null);
    }
  }, [loadDisplays]);

  const wakeAll = useCallback(async () => {
    setIsMutating(true);
    setPendingDisplayId(null);
    setState((current) => ({ ...current, feedback: null, error: null }));

    try {
      await withTimeout(invoke("unblank_all"), "Waking displays");
      setState((current) => ({
        ...current,
        feedback: "All displays are active again.",
      }));
    } catch (error) {
      setState((current) => ({
        ...current,
        error: error instanceof Error ? error.message : String(error),
      }));
      void loadDisplays(true);
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
    lastActiveDisplayId,
  };
}
