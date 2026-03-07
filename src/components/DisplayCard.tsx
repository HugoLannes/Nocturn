import type { Display } from "../types";

type DisplayCardProps = {
  display: Display;
  isMutating: boolean;
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
  isLastActiveDisplay,
  onToggle,
}: DisplayCardProps) {
  const isDisabled = isMutating || isLastActiveDisplay || !display.canBlackout;
  const name = cleanName(display.name);

  return (
    <button
      type="button"
      className={`display-card ${display.isBlackedOut ? "card-off" : "card-on"} ${isDisabled ? "card-disabled" : ""}`}
      onClick={() => onToggle(display.id)}
      disabled={isDisabled}
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
        <span className="state-label">{display.isBlackedOut ? "OFF" : "ON"}</span>
      </div>
    </button>
  );
}
