import type { Display } from "../types";

type DisplayCardProps = {
  display: Display;
  isMutating: boolean;
  isPending: boolean;
  isLastActiveDisplay: boolean;
  onToggle: (displayId: string) => void;
};

function cleanName(raw: string): string {
  const match = raw.match(/DISPLAY(\d+)/i);
  if (match) return `Display ${match[1]}`;
  return raw;
}

export function DisplayCard({
  display,
  isMutating,
  isPending,
  isLastActiveDisplay,
  onToggle,
}: DisplayCardProps) {
  const isDisabled = isMutating || isLastActiveDisplay || !display.canBlackout;
  const showDisabledState = (isLastActiveDisplay || !display.canBlackout) && !isPending;
  const name = cleanName(display.name);

  return (
    <button
      type="button"
      className={`display-card ${display.isBlackedOut ? "card-off" : "card-on"} ${showDisabledState ? "card-disabled" : ""} ${isPending ? "card-pending" : ""}`}
      onClick={() => onToggle(display.id)}
      disabled={isDisabled}
      aria-busy={isPending}
    >
      <div className="card-info">
        <div className="card-name-row">
          <span className="card-name">{name}</span>
          {display.isPrimary && <span className="card-badge">Primary</span>}
        </div>
        <span className="card-resolution">
          {display.width}×{display.height}
        </span>
      </div>

      <div className="card-state">
        <span className={`state-dot ${display.isBlackedOut ? "dot-off" : "dot-on"}`} />
        <span className="state-label">{isPending ? "..." : display.isBlackedOut ? "OFF" : "ON"}</span>
      </div>
    </button>
  );
}
