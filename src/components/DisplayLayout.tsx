import type { Display } from "../types";

type DisplayLayoutProps = {
  displays: Display[];
  headline: string;
  isMutating: boolean;
  pendingDisplayId: string | null;
  lastActiveDisplayId: string | null;
  onFocusMode: () => void;
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

function displayEyebrow(display: Display, index: number): string {
  return `Display ${shortLabel(display, index)}`;
}

function displayStateLabel(display: Display, isPending: boolean): string {
  if (isPending) return "Applying";
  if (display.isBlackedOut) return "Blacked out";
  if (!display.canBlackout) return "Protected";
  return "Active";
}

function displayTitle(display: Display): string {
  if (display.isPrimary) return "Primary workspace";
  if (display.hostsPanel) return "Nocturn panel";
  return cleanName(display.name);
}

function displaySummary(display: Display, isLastActive: boolean, isPending: boolean): string {
  if (isPending) return "Syncing display";
  if (display.isBlackedOut) return "Screen hidden";
  if (isLastActive) return "Last visible display";
  if (!display.canBlackout) return "Reserved by Nocturn";
  if (display.isPrimary) return "Main workspace";
  if (display.hostsPanel) return "Hosts the app";
  return "Ready to black out";
}

function displayActionLabel(display: Display, isLastActive: boolean, isPending: boolean): string {
  if (isPending) return "Applying...";
  if (display.isBlackedOut) return "Restore";
  if (isLastActive) return "Locked";
  if (!display.canBlackout) return "Unavailable";
  return "Black out";
}

function displayMeta(display: Display): string {
  return `${display.width}x${display.height}`;
}

function displayRoleTags(display: Display): string[] {
  return [
    display.isPrimary ? "Primary" : null,
    display.hostsPanel ? "Panel" : null,
  ].filter((tag): tag is string => Boolean(tag));
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

export function DisplayLayout({
  displays,
  headline,
  isMutating,
  pendingDisplayId,
  lastActiveDisplayId,
  onFocusMode,
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
  const activeCount = displays.filter((display) => !display.isBlackedOut).length;
  const primaryDisplay = displays.find((display) => display.isPrimary) ?? null;
  const activeSecondaryCount = displays.filter(
    (display) => !display.isPrimary && !display.isBlackedOut,
  ).length;
  const focusModeActive = activeCount === 1 && primaryDisplay !== null && !primaryDisplay.isBlackedOut;
  const canFocusMode = primaryDisplay !== null && (activeSecondaryCount > 0 || primaryDisplay.isBlackedOut);

  return (
    <section className="layout-panel" aria-label="Display arrangement">
      <header className="layout-header">
        <div className="layout-header-copy">
          <span className="layout-eyebrow">Display control</span>
          <h1 className="layout-title">{headline}</h1>
        </div>

        <button
          type="button"
          className={`focus-mode-btn ${focusModeActive ? "focus-mode-btn-active" : ""}`}
          onClick={onFocusMode}
          disabled={!canFocusMode || isMutating}
          aria-pressed={focusModeActive}
          aria-label="Enable focus mode and keep only the primary display active"
        >
          <span className="focus-mode-btn-label">Focus mode</span>
          <span className="focus-mode-btn-hint">Primary only</span>
        </button>
      </header>

      <div className="display-layout-frame" style={{ aspectRatio: String(mapAspectRatio) }}>
        <div className="display-layout-grid" aria-hidden="true" />

        <div className="display-layout-canvas">
          {displays.map((display, index) => {
            const isLastActive = lastActiveDisplayId === display.id && !display.isBlackedOut;
            const isDisabled = isMutating || isLastActive || !display.canBlackout;
            const isPending = pendingDisplayId === display.id;
            const left = ((display.x - minX) / totalWidth) * 100;
            const top = ((display.y - minY) / totalHeight) * 100;
            const width = (display.width / totalWidth) * 100;
            const height = (display.height / totalHeight) * 100;
            const isCompact = width < 30 || height < 42;
            const isTiny = width < 21 || height < 28;
            const stateLabel = displayStateLabel(display, isPending);
            const actionLabel = displayActionLabel(display, isLastActive, isPending);
            const roleTags = displayRoleTags(display);

            return (
              <button
                key={display.id}
                type="button"
                className={`layout-display ${display.isBlackedOut ? "layout-display-off" : "layout-display-on"} ${isPending ? "layout-display-pending" : ""} ${display.hostsPanel ? "layout-display-panel" : ""} ${isCompact ? "layout-display-compact" : ""} ${isTiny ? "layout-display-tiny" : ""}`}
                style={{ left: `${left}%`, top: `${top}%`, width: `${width}%`, height: `${height}%` }}
                onClick={() => onToggle(display.id)}
                disabled={isDisabled}
                aria-label={`${actionLabel} ${cleanName(display.name)} at ${display.width}x${display.height}`}
                aria-busy={isPending}
                title={`${cleanName(display.name)} • ${display.width}x${display.height} • ${display.x}, ${display.y}`}
              >
                <span className="layout-display-topline">
                  <span className="layout-display-eyebrow">{displayEyebrow(display, index)}</span>
                  <span className="layout-display-state-pill">
                    <span className="layout-display-state-dot" aria-hidden="true" />
                    <span className="layout-display-state">{stateLabel}</span>
                  </span>
                </span>
                <span className="layout-display-body">
                  <span className="layout-display-name">{displayTitle(display)}</span>
                  <span className="layout-display-summary">{displaySummary(display, isLastActive, isPending)}</span>
                </span>
                <span className="layout-display-bottomline">
                  <span className="layout-display-meta">{displayMeta(display)}</span>
                  <span className="layout-display-tags">
                    {!isTiny && roleTags.map((tag) => (
                      <span key={tag} className="layout-display-badge">
                        {tag}
                      </span>
                    ))}
                    <span className="layout-display-action">{actionLabel}</span>
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
