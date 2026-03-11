import type { CSSProperties } from "react";
import { formatShortcutForDisplay } from "../shortcuts";
import type { Display } from "../types";
import { ShortcutKeycaps } from "./ShortcutKeycaps";
import {
  cn,
  layoutEyebrowClass,
  layoutTitleClass,
  monoTextStyle,
} from "../ui";

type DisplayLayoutProps = {
  displays: Display[];
  headline: string;
  isMutating: boolean;
  pendingDisplayIds: ReadonlySet<string>;
  lastActiveDisplayId: string | null;
  focusModeHotkey: string | null;
  onFocusMode: () => void;
  onToggle: (displayId: string) => void;
};

const displayFrameStyle = {
  background:
    "linear-gradient(180deg, rgba(255, 255, 255, 0.035), rgba(255, 255, 255, 0.02)), rgba(255, 255, 255, 0.025)",
} satisfies CSSProperties;

const displayGridStyle = {
  backgroundImage:
    "linear-gradient(rgba(255, 255, 255, 0.028) 1px, transparent 1px), linear-gradient(90deg, rgba(255, 255, 255, 0.028) 1px, transparent 1px)",
  backgroundSize: "28px 28px",
  maskImage: "linear-gradient(180deg, rgba(0, 0, 0, 0.85), transparent 100%)",
  WebkitMaskImage: "linear-gradient(180deg, rgba(0, 0, 0, 0.85), transparent 100%)",
} satisfies CSSProperties;

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

function displayTitle(display: Display): string {
  return display.manufacturer || display.name || "Display";
}

function displaySummary(display: Display): string {
  return display.model || `${display.width}×${display.height}`;
}

function orientationLabel(orientation: number): string | null {
  switch (orientation) {
    case 1: return "90°";
    case 2: return "180°";
    case 3: return "270°";
    default: return null;
  }
}

function displayMeta(display: Display): string {
  const parts = [`${display.width}x${display.height}`];
  if (display.refreshRate > 1) parts.push(`${display.refreshRate}Hz`);
  const rotation = orientationLabel(display.orientation);
  if (rotation) parts.push(rotation);
  return parts.join(" · ");
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

function focusModeButtonClass(isActive: boolean) {
  return cn(
    "flex min-w-[168px] flex-col items-start gap-1 rounded-[14px] border px-[14px] py-[10px] text-[var(--text-primary)] transition-[border-color,background,box-shadow,transform,opacity] duration-[140ms] ease-out enabled:hover:-translate-y-px enabled:hover:border-[rgba(var(--accent-rgb),0.34)] enabled:hover:[background:linear-gradient(180deg,rgba(var(--accent-rgb),0.2),rgba(255,255,255,0.04)_46%),rgba(255,255,255,0.04)] enabled:hover:shadow-[inset_0_1px_0_rgba(255,255,255,0.06),0_8px_24px_rgba(var(--accent-rgb),0.18)] disabled:cursor-not-allowed disabled:opacity-40 max-[560px]:min-w-0 max-[560px]:w-full",
    isActive
      ? "border-[rgba(var(--accent-rgb),0.44)] [background:linear-gradient(180deg,rgba(var(--accent-rgb),0.24),rgba(var(--accent-rgb),0.08)_58%),rgba(255,255,255,0.04)] shadow-[inset_0_1px_0_rgba(255,255,255,0.08),0_10px_28px_rgba(var(--accent-rgb),0.22)]"
      : "border-white/10 [background:linear-gradient(180deg,rgba(var(--accent-rgb),0.14),rgba(255,255,255,0.03)_46%),rgba(255,255,255,0.03)]",
  );
}

function displayCardStyle(display: Display, isPending: boolean, left: number, top: number, width: number, height: number): CSSProperties {
  return {
    left: `${left}%`,
    top: `${top}%`,
    width: `${width}%`,
    height: `${height}%`,
    background: display.isBlackedOut
      ? "linear-gradient(180deg, rgba(248, 113, 113, 0.1), rgba(255, 255, 255, 0.015) 30%), linear-gradient(180deg, rgba(25, 20, 22, 0.96), rgba(11, 11, 15, 0.94))"
      : "linear-gradient(180deg, rgba(var(--accent-rgb), 0.18), rgba(255, 255, 255, 0.015) 30%), linear-gradient(180deg, rgba(20, 19, 28, 0.96), rgba(11, 12, 18, 0.92))",
    boxShadow: isPending
      ? "inset 0 1px 0 rgba(255, 255, 255, 0.1), 0 0 0 1.5px rgba(255, 255, 255, 0.2), 0 0 0 4px rgba(255, 255, 255, 0.04), 0 12px 28px rgba(2, 6, 23, 0.28)"
      : "inset 0 1px 0 rgba(255, 255, 255, 0.06), 0 10px 24px rgba(2, 6, 23, 0.18)",
  };
}

export function DisplayLayout({
  displays,
  headline,
  isMutating,
  pendingDisplayIds,
  lastActiveDisplayId,
  focusModeHotkey,
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
  const canFocusMode = primaryDisplay !== null
    && (focusModeActive || activeSecondaryCount > 0 || primaryDisplay.isBlackedOut);

  return (
    <section className="flex h-full min-h-0 flex-col gap-[10px]" aria-label="Display arrangement">
      <header className="flex items-center justify-between gap-[14px] px-[2px] pt-[2px] max-[560px]:flex-col max-[560px]:items-stretch">
        <div className="flex max-w-[460px] flex-col gap-[3px]">
          <span className={layoutEyebrowClass} style={monoTextStyle}>Display control</span>
          <h1 className={layoutTitleClass}>{headline}</h1>
        </div>

        <button
          type="button"
          className={focusModeButtonClass(focusModeActive)}
          onClick={onFocusMode}
          disabled={!canFocusMode || isMutating}
          aria-pressed={focusModeActive}
          aria-label={focusModeActive
            ? "Disable focus mode and restore all displays"
            : "Enable focus mode and keep only the primary display active"}
        >
          <span className="text-[14px] font-semibold leading-[1.05] tracking-[-0.03em]">Focus mode</span>
          <span className="flex flex-wrap items-center gap-x-[6px] gap-y-[4px]">
            <span className="text-[10px] uppercase leading-[1.2] tracking-[0.08em] text-[rgba(226,232,240,0.64)]" style={monoTextStyle}>
              {focusModeActive ? "Restore all" : "Primary only"}
            </span>
            <ShortcutKeycaps accelerator={focusModeHotkey} size="sm" />
          </span>
        </button>
      </header>

      <div
        className="relative min-h-0 flex-1 overflow-hidden rounded-[14px] border border-[var(--border)] p-[10px] max-[560px]:p-2"
        style={{ ...displayFrameStyle, aspectRatio: String(mapAspectRatio) }}
      >
        <div className="absolute inset-0" style={displayGridStyle} aria-hidden="true" />

        <div className="relative h-full w-full">
          {displays.map((display, index) => {
            const isLastActive = lastActiveDisplayId === display.id && !display.isBlackedOut;
            const isPending = pendingDisplayIds.has(display.id);
            const shortcutLabel = formatShortcutForDisplay(display.hotkey);
            // isMutating is only true during global ops (focus mode, restore all).
            // Individual card toggles use per-card isPending so other cards stay interactive.
            const isDisabled = isMutating || isPending || isLastActive || !display.canBlackout;
            const left = ((display.x - minX) / totalWidth) * 100;
            const top = ((display.y - minY) / totalHeight) * 100;
            const width = (display.width / totalWidth) * 100;
            const height = (display.height / totalHeight) * 100;
            const isCompact = width < 30 || height < 42;
            const isTiny = width < 21 || height < 28;
            const roleTags = displayRoleTags(display);
            const eyebrowSizeClass = isCompact ? "text-[8px]" : "text-[9px]";
            const nameSizeClass = isTiny ? "text-[10px]" : isCompact ? "text-[11px]" : "text-[12px]";
            const metaSizeClass = isCompact ? "text-[7px]" : "text-[8px]";
            const badgeSizeClass = isCompact ? "text-[6px]" : "text-[7px]";

            return (
              <button
                key={display.id}
                type="button"
                className={cn(
                  "absolute grid min-h-[72px] min-w-[92px] cursor-pointer grid-rows-[auto_1fr_auto] gap-[6px] overflow-hidden rounded-[14px] border p-[10px] text-left text-[#f5f7fb] backdrop-blur-[12px] transition-[border-color,box-shadow,opacity,transform] duration-120 ease-out enabled:hover:border-white/18 enabled:hover:shadow-[inset_0_1px_0_rgba(255,255,255,0.08),0_14px_30px_rgba(2,6,23,0.22),0_0_0_1px_rgba(255,255,255,0.025)] enabled:active:scale-[0.99] disabled:pointer-events-none disabled:opacity-[0.58] max-[560px]:min-h-[68px] max-[560px]:min-w-[84px] max-[560px]:p-2",
                  isPending
                    ? "border-white/20"
                    : display.isBlackedOut
                      ? "border-[rgba(248,113,113,0.2)]"
                      : "border-[rgba(var(--accent-rgb),0.26)]",
                  isCompact && "gap-[5px] p-2",
                  isTiny && "gap-1 p-[7px]",
                )}
                style={displayCardStyle(display, isPending, left, top, width, height)}
                onClick={() => onToggle(display.id)}
                disabled={isDisabled}
                aria-label={`${cleanName(display.name)} at ${display.width}x${display.height}${shortcutLabel ? `, shortcut ${shortcutLabel}` : ""}`}
                aria-busy={isPending}
                title={`${cleanName(display.name)} • ${display.width}x${display.height} • ${display.x}, ${display.y}${shortcutLabel ? ` • ${shortcutLabel}` : ""}`}
              >
                <span className={cn("flex items-center justify-between gap-[5px]", isTiny && "items-start")}>
                  <span className={cn(layoutEyebrowClass, eyebrowSizeClass)} style={monoTextStyle}>
                    {displayEyebrow(display, index)}
                  </span>
                  <span
                    className={cn(
                      "h-[7px] w-[7px] shrink-0 rounded-full transition-[background-color,box-shadow] duration-120",
                      display.isBlackedOut
                        ? "bg-[rgba(248,113,113,0.9)] shadow-[0_0_6px_rgba(248,113,113,0.7),0_0_12px_rgba(248,113,113,0.35)]"
                        : "bg-[rgba(var(--accent-rgb),0.9)] shadow-[0_0_6px_rgba(var(--accent-rgb),0.7),0_0_12px_rgba(var(--accent-rgb),0.35)]",
                      isPending && "animate-pulse",
                    )}
                    aria-hidden="true"
                  />
                </span>
                <span className="flex min-w-0 flex-col items-start justify-center gap-1">
                  <span className={cn("max-w-full overflow-hidden text-ellipsis whitespace-nowrap font-bold leading-[1.05] tracking-[-0.035em]", nameSizeClass)}>
                    {displayTitle(display)}
                  </span>
                  {!isTiny && (!isCompact || !shortcutLabel) ? (
                    <span
                      className={cn(
                        "overflow-hidden leading-[1.35] text-[rgba(226,232,240,0.72)]",
                        isCompact ? "truncate text-[9px]" : "text-[10px] [display:-webkit-box] [-webkit-box-orient:vertical] [-webkit-line-clamp:2]",
                      )}
                    >
                      {displaySummary(display)}
                    </span>
                  ) : null}
                  {shortcutLabel ? (
                    <ShortcutKeycaps
                      accelerator={display.hotkey}
                      size={isTiny || isCompact ? "xs" : "sm"}
                      className="max-w-full"
                    />
                  ) : null}
                </span>

                <span className={cn("flex items-center justify-between gap-[5px]", isTiny && "items-start")}>
                  {!isTiny ? (
                    <span className={cn("overflow-hidden text-ellipsis whitespace-nowrap text-[rgba(226,232,240,0.58)]", metaSizeClass)} style={monoTextStyle}>
                      {displayMeta(display)}
                    </span>
                  ) : <span />}

                  <span className="flex min-w-0 flex-wrap items-center justify-end gap-1">
                    {!isTiny && roleTags.map((tag) => (
                      <span
                        key={tag}
                        className={cn(
                          "rounded-full border border-[rgba(var(--accent-rgb),0.28)] px-[5px] py-[2px] font-semibold uppercase tracking-[0.08em] text-[var(--accent-soft)] shadow-[inset_0_1px_0_rgba(255,255,255,0.08)] [background:linear-gradient(180deg,rgba(var(--accent-rgb),0.24),rgba(var(--accent-rgb),0.08))]",
                          badgeSizeClass,
                        )}
                        style={monoTextStyle}
                      >
                        {tag}
                      </span>
                    ))}
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
