import { useEffect, useRef, useState, type KeyboardEvent as ReactKeyboardEvent } from "react";
import { buildAcceleratorFromKeyboardEvent, formatShortcutForDisplay, isModifierKey } from "../shortcuts";
import { cn, monoTextStyle } from "../ui";

type ShortcutFieldProps = {
  title: string;
  hint: string;
  value: string | null;
  disabled: boolean;
  statusText?: string;
  onSubmit: (accelerator: string | null) => Promise<string | null>;
};

const triggerButtonClass = "inline-flex min-w-[156px] items-center justify-center rounded-[12px] border px-3 py-2 text-[13px] font-semibold leading-none tracking-[-0.02em] transition-[border-color,background,box-shadow,transform,opacity] duration-[140ms] ease-out enabled:hover:-translate-y-px disabled:cursor-not-allowed disabled:opacity-50 max-[560px]:w-full";
const actionButtonClass = "inline-flex items-center justify-center rounded-[12px] border border-white/10 bg-white/[0.03] px-3 py-2 text-[12px] font-medium text-[var(--text-secondary)] transition-[border-color,background,color] duration-[140ms] ease-out enabled:hover:border-white/16 enabled:hover:bg-white/[0.06] enabled:hover:text-[var(--text-primary)] disabled:cursor-not-allowed disabled:opacity-45 max-[560px]:flex-1";

export function ShortcutField({
  title,
  hint,
  value,
  disabled,
  statusText,
  onSubmit,
}: ShortcutFieldProps) {
  const triggerRef = useRef<HTMLButtonElement | null>(null);
  const [isCapturing, setIsCapturing] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  useEffect(() => {
    if (isCapturing) {
      triggerRef.current?.focus();
    }
  }, [isCapturing]);

  useEffect(() => {
    setErrorMessage(null);
  }, [value]);

  async function submitShortcut(accelerator: string | null) {
    setIsSaving(true);
    const error = await onSubmit(accelerator);
    setIsSaving(false);

    if (error) {
      setErrorMessage(error);
      return;
    }

    setErrorMessage(null);
    setIsCapturing(false);
  }

  function startCapture() {
    if (disabled || isSaving) {
      return;
    }

    setErrorMessage(null);
    setIsCapturing(true);
  }

  async function handleKeyDown(event: ReactKeyboardEvent<HTMLButtonElement>) {
    if (!isCapturing || disabled || isSaving) {
      return;
    }

    event.preventDefault();
    event.stopPropagation();

    if (event.key === "Escape") {
      setIsCapturing(false);
      setErrorMessage(null);
      return;
    }

    if (
      (event.key === "Backspace" || event.key === "Delete")
      && !event.altKey
      && !event.ctrlKey
      && !event.metaKey
      && !event.shiftKey
    ) {
      await submitShortcut(null);
      return;
    }

    if (isModifierKey(event.key)) {
      return;
    }

    const accelerator = buildAcceleratorFromKeyboardEvent(event.nativeEvent);
    if (!accelerator) {
      setErrorMessage("Use at least one modifier key.");
      return;
    }

    await submitShortcut(accelerator);
  }

  async function handleClear() {
    if (!value || disabled || isSaving) {
      return;
    }

    await submitShortcut(null);
  }

  const triggerLabel = isCapturing
    ? "Press a shortcut"
    : value
      ? formatShortcutForDisplay(value)
      : "No shortcut set";
  const isDisabled = disabled || isSaving;

  return (
    <div className="flex items-center justify-between gap-4 rounded-[14px] border border-white/8 bg-white/[0.025] px-4 py-3 max-[560px]:flex-col max-[560px]:items-stretch max-[560px]:px-3">
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          <span className="text-[14px] font-semibold leading-[1.15]">{title}</span>
          {statusText ? (
            <span
              className="rounded-full border border-white/10 bg-white/[0.04] px-2 py-[3px] text-[9px] uppercase tracking-[0.08em] text-[rgba(226,232,240,0.7)]"
              style={monoTextStyle}
            >
              {statusText}
            </span>
          ) : null}
        </div>
        <p className="mt-1.5 text-[13px] leading-[1.6] text-[rgba(226,232,240,0.66)]">{hint}</p>
        {errorMessage ? (
          <p className="mt-2 text-[12px] leading-[1.5] text-[#fca5a5]">{errorMessage}</p>
        ) : null}
      </div>

      <div className="flex shrink-0 items-center gap-2 max-[560px]:w-full max-[560px]:flex-wrap">
        <button
          ref={triggerRef}
          type="button"
          className={cn(
            triggerButtonClass,
            isCapturing
              ? "border-[rgba(var(--accent-rgb),0.42)] bg-[rgba(var(--accent-rgb),0.18)] text-[var(--accent-soft)] shadow-[0_0_0_1px_rgba(var(--accent-rgb),0.18)]"
              : value
                ? "border-[rgba(var(--accent-rgb),0.24)] bg-[rgba(var(--accent-rgb),0.12)] text-[var(--text-primary)]"
                : "border-white/10 bg-white/[0.03] text-[var(--text-secondary)]",
          )}
          onClick={startCapture}
          onKeyDown={(event) => void handleKeyDown(event)}
          onBlur={() => {
            if (!isSaving) {
              setIsCapturing(false);
            }
          }}
          aria-label={`Set shortcut for ${title}`}
          aria-pressed={isCapturing}
          disabled={isDisabled}
        >
          <span style={monoTextStyle}>{triggerLabel}</span>
        </button>

        <button
          type="button"
          className={actionButtonClass}
          onClick={() => void handleClear()}
          disabled={isDisabled || !value}
        >
          Clear
        </button>
      </div>
    </div>
  );
}
