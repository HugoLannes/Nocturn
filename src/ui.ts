import type { CSSProperties } from "react";

export function cn(...classes: Array<string | false | null | undefined>) {
  return classes.filter(Boolean).join(" ");
}

export const monoTextStyle = {
  fontFamily: '"IBM Plex Mono", monospace',
} satisfies CSSProperties;

export const layoutEyebrowClass = "text-[11px] uppercase tracking-[0.1em] text-[rgba(226,232,240,0.62)]";

export const layoutTitleClass = "overflow-hidden whitespace-nowrap text-ellipsis text-[24px] font-semibold leading-none tracking-[-0.03em] text-[var(--text-primary)] max-[560px]:text-[20px]";

export const displayStatePillClass = "flex items-center gap-1";

export const displayStateDotClass = "h-1.5 w-1.5 rounded-full bg-current opacity-90 shadow-[0_0_10px_currentColor]";

export const displayStateTextClass = "whitespace-nowrap text-[7px] font-semibold uppercase tracking-[0.08em] text-[rgba(255,255,255,0.72)]";
