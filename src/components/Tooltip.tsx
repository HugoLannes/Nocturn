import { cloneElement, isValidElement, useId, type CSSProperties, type ReactElement } from "react";
import { cn } from "../ui";

type TooltipProps = {
  title: string;
  description?: string;
  side?: "top" | "bottom";
  children: ReactElement<{ "aria-describedby"?: string }>;
};

const tooltipBubbleStyle = {
  background:
    "linear-gradient(180deg, rgba(255, 255, 255, 0.12) 0%, rgba(255, 255, 255, 0.04) 100%), rgba(11, 14, 22, 0.92)",
  boxShadow:
    "0 16px 40px rgba(0, 0, 0, 0.38), 0 0 0 1px rgba(255, 255, 255, 0.04), 0 0 24px rgba(var(--accent-rgb), 0.12)",
  backdropFilter: "blur(18px) saturate(140%)",
  WebkitBackdropFilter: "blur(18px) saturate(140%)",
} satisfies CSSProperties;

const tooltipBorderClass = "pointer-events-none absolute inset-0 rounded-[inherit] p-px opacity-95 [background:linear-gradient(135deg,rgba(255,255,255,0.04)_0%,rgba(255,255,255,0.02)_22%,rgba(var(--accent-rgb),0.05)_38%,rgba(var(--accent-rgb),0.24)_52%,rgba(var(--accent-rgb),0.14)_68%,rgba(255,255,255,0.03)_82%,rgba(255,255,255,0.02)_100%)] [mask:linear-gradient(#fff_0_0)_content-box,linear-gradient(#fff_0_0)] [-webkit-mask:linear-gradient(#fff_0_0)_content-box,linear-gradient(#fff_0_0)] [mask-composite:exclude] [-webkit-mask-composite:xor]";

const tooltipSheenClass = "pointer-events-none absolute inset-px rounded-[inherit] [background:radial-gradient(circle_at_top,rgba(255,255,255,0.15),transparent_54%),linear-gradient(135deg,rgba(var(--accent-rgb),0.16),transparent_44%)] opacity-80";

const tooltipAccentClass = "pointer-events-none absolute inset-0 rounded-[inherit] shadow-[inset_0_0_0_1px_rgba(var(--accent-rgb),0.06),inset_0_1px_0_rgba(255,255,255,0.05)]";

export function Tooltip({
  title,
  description,
  side = "top",
  children,
}: TooltipProps) {
  const tooltipId = useId();

  if (!isValidElement(children)) {
    return null;
  }

  const childAriaDescribedBy = children.props["aria-describedby"];
  const describedBy = childAriaDescribedBy
    ? `${childAriaDescribedBy} ${tooltipId}`
    : tooltipId;
  const bubblePositionClass = side === "top"
    ? "bottom-[calc(100%+12px)] -translate-x-1/2 translate-y-[6px] motion-reduce:translate-y-0"
    : "top-[calc(100%+12px)] -translate-x-1/2 -translate-y-[6px] motion-reduce:translate-y-0";

  return (
    <span className="group relative isolate inline-flex">
      {cloneElement(children, {
        "aria-describedby": describedBy,
      })}

      <span
        id={tooltipId}
        role="tooltip"
        className={cn(
          "pointer-events-none absolute left-1/2 z-30 min-w-[188px] max-w-[240px] rounded-[14px] border border-white/8 px-3 py-[10px] opacity-0 invisible scale-[0.98] transition-[opacity,transform,visibility] duration-[180ms] ease-[cubic-bezier(0.22,1,0.36,1)] group-hover:visible group-hover:opacity-100 group-hover:scale-100 group-focus-within:visible group-focus-within:opacity-100 group-focus-within:scale-100 motion-reduce:scale-100 motion-reduce:transition-[opacity,visibility] motion-reduce:duration-[120ms]",
          bubblePositionClass,
        )}
        style={tooltipBubbleStyle}
      >
        <span aria-hidden="true" className={tooltipBorderClass} />
        <span aria-hidden="true" className={tooltipSheenClass} />
        <span className="relative flex flex-col gap-[3px]">
          <span className="text-[12px] font-semibold tracking-[0.01em] text-[#f8fafc]">{title}</span>
          {description ? (
            <span className="text-[11px] leading-[1.4] text-[rgba(226,232,240,0.72)]">{description}</span>
          ) : null}
        </span>
        <span aria-hidden="true" className={tooltipAccentClass} />
      </span>
    </span>
  );
}
