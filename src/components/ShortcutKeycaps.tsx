import { formatShortcutTokensForDisplay } from "../shortcuts";
import { cn, monoTextStyle } from "../ui";

type ShortcutKeycapsProps = {
  accelerator: string | null | undefined;
  size?: "sm" | "xs";
  className?: string;
};

const groupGapClass = {
  sm: "gap-[3px]",
  xs: "gap-[2px]",
} as const;

const tokenGapClass = {
  sm: "gap-[3px]",
  xs: "gap-[2px]",
} as const;

const separatorClass = {
  sm: "text-[9px]",
  xs: "text-[7px]",
} as const;

const keycapClass = {
  sm: "min-w-[18px] h-[18px] px-[5px] text-[9px]",
  xs: "min-w-[14px] h-[15px] px-[4px] text-[7px]",
} as const;

export function ShortcutKeycaps({ accelerator, size = "sm", className }: ShortcutKeycapsProps) {
  const tokens = formatShortcutTokensForDisplay(accelerator);

  if (tokens.length === 0) {
    return null;
  }

  return (
    <span
      aria-hidden="true"
      className={cn("flex max-w-full flex-wrap items-center normal-case", groupGapClass[size], className)}
    >
      {tokens.map((token, index) => (
        <span key={`${token}-${index}`} className={cn("inline-flex items-center", tokenGapClass[size])}>
          {index > 0 ? (
            <span className={cn("text-[rgba(226,232,240,0.52)]", separatorClass[size])} style={monoTextStyle}>
              +
            </span>
          ) : null}
          <span
            className={cn(
              "inline-flex items-center justify-center whitespace-nowrap rounded-[4px] border border-white/10 text-[rgba(248,250,252,0.9)] shadow-[inset_0_1px_0_rgba(255,255,255,0.08),inset_0_-1px_0_rgba(8,10,14,0.36),0_1px_1px_rgba(8,10,14,0.18)] [background:linear-gradient(180deg,rgba(255,255,255,0.14),rgba(255,255,255,0.04)_62%),rgba(148,163,184,0.05)]",
              keycapClass[size],
            )}
            style={monoTextStyle}
          >
            {token}
          </span>
        </span>
      ))}
    </span>
  );
}
