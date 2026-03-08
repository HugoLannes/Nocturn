import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useEffect, useMemo, useState, type CSSProperties } from "react";
import type { HiddenAppSummary, OverlayCardPresentation } from "./types";
import {
  displayStateDotClass,
  displayStatePillClass,
  displayStateTextClass,
  layoutEyebrowClass,
  monoTextStyle,
} from "./ui";

const MAX_OVERLAY_ROWS = 4;

type OverlayRow =
  | { kind: "app"; appName: string; windowCount: number }
  | { kind: "message"; label: string };

const overlayShellStyle = {
  background:
    "linear-gradient(180deg, rgba(var(--accent-rgb), 0.18), rgba(255, 255, 255, 0.015) 30%), linear-gradient(180deg, rgba(20, 19, 28, 0.96), rgba(11, 12, 18, 0.92))",
  boxShadow: "inset 0 1px 0 rgba(255, 255, 255, 0.06), 0 10px 24px rgba(2, 6, 23, 0.18)",
} satisfies CSSProperties;

const overlayIconStyle = {
  background: "linear-gradient(180deg, rgba(var(--accent-rgb), 0.94), rgba(var(--accent-rgb), 0.76))",
  boxShadow:
    "inset 0 1px 0 rgba(255, 255, 255, 0.08), 0 0 0 1px rgba(var(--accent-rgb), 0.24), 0 4px 18px rgba(var(--accent-rgb), 0.28)",
} satisfies CSSProperties;

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
    return <div className="flex h-full w-full items-stretch bg-transparent" />;
  }

  const rows = buildRows(presentation.hiddenApps);
  const maxIconSlots = rows.reduce(
    (max, row) => (row.kind === "app" ? Math.max(max, row.windowCount) : max),
    1,
  );
  const iconRailWidth = maxIconSlots * 5 + Math.max(maxIconSlots - 1, 0) * 4;
  const shellStyle = {
    ...overlayShellStyle,
    "--overlay-icon-rail-width": `${iconRailWidth}px`,
  } as CSSProperties;

  const appCount = presentation.hiddenApps.length;

  return (
    <div className={`flex h-full w-full items-stretch bg-transparent overlay-card-window-${presentation.dock}`}>
      <div
        className="relative flex-1 overflow-hidden  border border-[rgba(var(--accent-rgb),0.26)] p-[10px] text-left text-[#f5f7fb] backdrop-blur-[12px]"
        style={shellStyle}
      >
        <div className="grid h-full grid-rows-[auto_1fr] gap-[6px]">
          <span className="flex items-center justify-between gap-[5px]">
            <span className={layoutEyebrowClass} style={monoTextStyle}>Hidden apps</span>
            <span className={`${displayStatePillClass} text-[#c7bcff]`}>
              <span className={displayStateDotClass} aria-hidden="true" />
              <span className={displayStateTextClass}>{appCount === 0 ? "Empty" : `${appCount} app${appCount > 1 ? "s" : ""}`}</span>
            </span>
          </span>

          <div className="flex min-w-0 flex-col">
            {rows.map((row, index) => (
              row.kind === "app" ? (
                <div
                  key={`${row.appName}-${index}`}
                  className="relative grid min-h-[36px] items-center gap-x-[10px] px-1 [grid-template-columns:var(--overlay-icon-rail-width,5px)_minmax(0,1fr)]"
                  style={{ borderBottom: index === rows.length - 1 ? "0" : "1px solid rgba(255, 255, 255, 0.06)" }}
                >
                  <div className="flex min-w-2 shrink-0 items-center gap-1" aria-hidden="true">
                    {Array.from({ length: row.windowCount }).map((_, iconIndex) => (
                      <span
                        key={iconIndex}
                        className="h-[14px] w-[5px] rounded-full"
                        style={overlayIconStyle}
                      />
                    ))}
                  </div>
                  <span className="min-w-0 overflow-hidden text-ellipsis whitespace-nowrap text-[12px] font-bold leading-[1.05] tracking-[-0.035em] text-[#f5f7fb]">
                    {row.appName}
                  </span>
                </div>
              ) : (
                <div
                  key={`${row.label}-${index}`}
                  className="relative flex min-h-[36px] justify-center px-1 pt-[2px]"
                  style={{ borderBottom: index === rows.length - 1 ? "0" : "1px solid rgba(255, 255, 255, 0.06)" }}
                >
                  <span className="text-[8px] font-semibold uppercase tracking-[0.08em] text-[rgba(226,232,240,0.68)]" style={monoTextStyle}>
                    {row.label}
                  </span>
                </div>
              )
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
