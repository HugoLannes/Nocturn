import type { Display } from "../types";

type DisplayLayoutProps = {
  displays: Display[];
  isMutating: boolean;
  pendingDisplayId: string | null;
  lastActiveDisplayId: string | null;
  onToggle: (displayId: string) => void;
};

function cleanName(raw: string): string {
  const match = raw.match(/DISPLAY(\d+)/i);
  if (match) return `Display ${match[1]}`;
  return raw;
}

function shortLabel(display: Display, index: number): string {
  const match = display.name.match(/DISPLAY(\d+)/i);
  if (match) return match[1];
  return String(index + 1);
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

export function DisplayLayout({
  displays,
  isMutating,
  pendingDisplayId,
  lastActiveDisplayId,
  onToggle,
}: DisplayLayoutProps) {
  if (displays.length === 0) {
    return null;
  }

  const minX = Math.min(...displays.map((display) => display.x));
  const minY = Math.min(...displays.map((display) => display.y));
  const maxX = Math.max(...displays.map((display) => display.x + display.width));
  const maxY = Math.max(...displays.map((display) => display.y + display.height));
  const totalWidth = Math.max(maxX - minX, 1);
  const totalHeight = Math.max(maxY - minY, 1);
  const mapAspectRatio = clamp(totalWidth / totalHeight, 1.1, 2.2);

  return (
    <section className="layout-panel" aria-label="Display arrangement">
      <div className="display-layout-frame" style={{ aspectRatio: String(mapAspectRatio) }}>
        <div className="display-layout-grid" aria-hidden="true" />

        <div className="display-layout-canvas">
          {displays.map((display, index) => {
            const isLastActive = lastActiveDisplayId === display.id && !display.isBlackedOut;
            const isDisabled = isMutating || isLastActive || !display.canBlackout;
            const left = ((display.x - minX) / totalWidth) * 100;
            const top = ((display.y - minY) / totalHeight) * 100;
            const width = (display.width / totalWidth) * 100;
            const height = (display.height / totalHeight) * 100;

            return (
              <button
                key={display.id}
                type="button"
                className={`layout-display ${display.isBlackedOut ? "layout-display-off" : "layout-display-on"} ${pendingDisplayId === display.id ? "layout-display-pending" : ""} ${display.hostsPanel ? "layout-display-panel" : ""}`}
                style={{ left: `${left}%`, top: `${top}%`, width: `${width}%`, height: `${height}%` }}
                onClick={() => onToggle(display.id)}
                disabled={isDisabled}
                aria-label={`Toggle ${cleanName(display.name)} at ${display.width}x${display.height}`}
                aria-busy={pendingDisplayId === display.id}
              >
                <span className="layout-display-topline">
                  <span className="layout-display-index">{shortLabel(display, index)}</span>
                  <span className="layout-display-state-pill">
                    <span className="layout-display-state-dot" aria-hidden="true" />
                    <span className="layout-display-state">{pendingDisplayId === display.id ? "Sync" : display.isBlackedOut ? "Off" : "On"}</span>
                  </span>
                </span>
                <span className="layout-display-body">
                  <span className="layout-display-name">{cleanName(display.name)}</span>
                  <span className="layout-display-meta">
                    {display.width}x{display.height}
                  </span>
                </span>
                <span className="layout-display-bottomline">
                  <span className="layout-display-coords">
                    {display.x}, {display.y}
                  </span>
                  <span className="layout-display-tags">
                    {display.isPrimary && <span className="layout-display-badge">Primary</span>}
                  </span>
                </span>
              </button>
            );
          })}
        </div>
      </div>
    </section>
  );
}
