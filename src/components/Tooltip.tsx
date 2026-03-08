import { cloneElement, isValidElement, useId, type ReactElement } from "react";

type TooltipProps = {
  title: string;
  description?: string;
  side?: "top" | "bottom";
  children: ReactElement<{ "aria-describedby"?: string }>;
};

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

  return (
    <span className={`tooltip-root tooltip-side-${side}`}>
      {cloneElement(children, {
        "aria-describedby": describedBy,
      })}

      <span id={tooltipId} role="tooltip" className="tooltip-bubble">
        <span className="tooltip-sheen" aria-hidden="true" />
        <span className="tooltip-copy">
          <span className="tooltip-title">{title}</span>
          {description ? <span className="tooltip-description">{description}</span> : null}
        </span>
        <span className="tooltip-accent" aria-hidden="true" />
      </span>
    </span>
  );
}
