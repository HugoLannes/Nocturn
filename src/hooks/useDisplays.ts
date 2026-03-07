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

export function useDisplays() {
  const [state, setState] = useState<UseDisplaysState>(INITIAL_STATE);
  const [isMutating, setIsMutating] = useState(false);

  const loadDisplays = useCallback(async (preserveError = false) => {
    try {
      const payload = await invoke<DisplayUpdatePayload>("get_displays");
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
    let hadError = false;
    setIsMutating(true);
    setState((current) => ({ ...current, feedback: null, error: null }));

    try {
      const feedback = await invoke<string>("toggle_display", { id: displayId });
      if (feedback) {
        setState((current) => ({ ...current, feedback }));
      }
    } catch (error) {
      setState((current) => ({
        ...current,
        error: error instanceof Error ? error.message : String(error),
      }));
      hadError = true;
    } finally {
      setIsMutating(false);
      await loadDisplays(hadError);
    }
  }, [loadDisplays]);

  const wakeAll = useCallback(async () => {
    let hadError = false;
    setIsMutating(true);
    setState((current) => ({ ...current, feedback: null, error: null }));

    try {
      await invoke("unblank_all");
      setState((current) => ({
        ...current,
        feedback: "All displays are active again.",
      }));
    } catch (error) {
      setState((current) => ({
        ...current,
        error: error instanceof Error ? error.message : String(error),
      }));
      hadError = true;
    } finally {
      setIsMutating(false);
      await loadDisplays(hadError);
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
    loadDisplays,
    toggleDisplay,
    wakeAll,
    lastActiveDisplayId,
  };
}
