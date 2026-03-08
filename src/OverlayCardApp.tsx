import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useEffect, useMemo, useState } from "react";
import type { CSSProperties } from "react";
import type { HiddenAppSummary, OverlayCardPresentation } from "./types";

const MAX_OVERLAY_ROWS = 4;

type OverlayRow =
  | { kind: "app"; appName: string; windowCount: number }
  | { kind: "message"; label: string };

function buildRows(hiddenApps: HiddenAppSummary[]): OverlayRow[] {
  if (hiddenApps.length === 0) {
    return [{ kind: "message", label: "No hidden apps on this display" }];
  }

  const rows: OverlayRow[] = hiddenApps.slice(0, MAX_OVERLAY_ROWS).map((app) => ({
    kind: "app",
    appName: app.appName,
    windowCount: Math.max(1, app.windowCount),
  }));

  if (hiddenApps.length > MAX_OVERLAY_ROWS) {
    rows.push({
      kind: "message",
      label: `And ${hiddenApps.length - MAX_OVERLAY_ROWS} more hidden apps`,
    });
  }

  return rows;
}

export function OverlayCardApp() {
  const currentWindow = useMemo(() => getCurrentWebviewWindow(), []);
  const windowLabel = currentWindow.label;
  const [presentation, setPresentation] = useState<OverlayCardPresentation | null>(null);

  useEffect(() => {
    let isMounted = true;

    void invoke<OverlayCardPresentation | null>("get_overlay_card_presentation", { windowLabel }).then((payload) => {
      if (isMounted) {
        setPresentation(payload);
      }
    });

    const unlistenPromise = listen<OverlayCardPresentation>("overlay-card:update", (event) => {
      setPresentation(event.payload);
    });

    return () => {
      isMounted = false;
      void unlistenPromise.then((unlisten) => unlisten());
    };
  }, [windowLabel]);

  if (!presentation?.isEnabled) {
    return <div className="overlay-card-window" />;
  }

  const rows = buildRows(presentation.hiddenApps);
  const maxIconSlots = rows.reduce(
    (max, row) => (row.kind === "app" ? Math.max(max, row.windowCount) : max),
    1,
  );
  const iconRailWidth = maxIconSlots * 5 + Math.max(maxIconSlots - 1, 0) * 4;
  const shellStyle = {
    "--overlay-icon-rail-width": `${iconRailWidth}px`,
  } as CSSProperties;

  return (
    <div className={`overlay-card-window overlay-card-window-${presentation.dock}`}>
      <div className="overlay-card-shell" style={shellStyle}>
        {rows.map((row, index) => (
          row.kind === "app" ? (
            <div key={`${row.appName}-${index}`} className="overlay-card-row">
              <div className="overlay-card-icons" aria-hidden="true">
                {Array.from({ length: row.windowCount }).map((_, iconIndex) => (
                  <span key={iconIndex} className="overlay-card-icon" />
                ))}
              </div>
              <span className="overlay-card-label">{row.appName}</span>
            </div>
          ) : (
            <div key={`${row.label}-${index}`} className="overlay-card-row overlay-card-row-muted">
              <span className="overlay-card-label overlay-card-muted-label">{row.label}</span>
            </div>
          )
        ))}
      </div>
    </div>
  );
}
