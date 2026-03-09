import { useEffect, useRef, useState, type KeyboardEvent as ReactKeyboardEvent } from "react";
import { buildAcceleratorFromKeyboardEvent, isModifierKey } from "../shortcuts";
import { cn, monoTextStyle } from "../ui";
import { ShortcutKeycaps } from "./ShortcutKeycaps";

type ShortcutFieldProps = {
  title: string;
  hint: string;
  value: string | null;
  disabled: boolean;
  statusText?: string;
  onSubmit: (accelerator: string | null) => Promise<string | null>;
};

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
  const [feedbackState, setFeedbackState] = useState<"idle" | "saved">("idle");

  useEffect(() => {
    if (isCapturing) {
      triggerRef.current?.focus();
    }
  }, [isCapturing]);

  useEffect(() => {
    setErrorMessage(null);
  }, [value]);

  useEffect(() => {
    if (feedbackState !== "saved") {
      return;
    }

    const timer = window.setTimeout(() => {
      setFeedbackState("idle");
    }, 1400);

    return () => window.clearTimeout(timer);
  }, [feedbackState]);

  async function submitShortcut(accelerator: string | null) {
    setIsSaving(true);
    setFeedbackState("idle");
    const error = await onSubmit(accelerator);
    setIsSaving(false);

    if (error) {
      setErrorMessage(error);
      setFeedbackState("idle");
      return;
    }

    setErrorMessage(null);
    setFeedbackState("saved");
    setIsCapturing(false);
  }

  function startCapture() {
    if (disabled || isSaving) {
      return;
    }

    setErrorMessage(null);
    setFeedbackState("idle");
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

  const isDisabled = disabled || isSaving;

  return (
    <div
      className={cn(
        "flex items-center justify-between gap-3 rounded-lg py-2.5 transition-[background] duration-[140ms] ease-out max-[560px]:flex-col max-[560px]:items-stretch",
        isCapturing && "bg-[rgba(var(--accent-rgb),0.06)]",
      )}
    >
      <div className="flex min-w-0 items-center gap-2">
        <span className="shrink-0 text-[13px] font-semibold leading-[1.15] text-[var(--text-primary)]">
          {title}
        </span>
        <span className="truncate text-[12px] text-[rgba(226,232,240,0.48)]">{hint}</span>
        {statusText && !isCapturing ? (
          <span className="shrink-0 text-[11px] text-[rgba(226,232,240,0.38)]" style={monoTextStyle}>
            {statusText}
          </span>
        ) : null}
      </div>

      <div className="flex shrink-0 items-center gap-2">
        {isCapturing ? (
          <>
            <span className="animate-pulse text-[12px] text-[var(--accent-soft)]" style={monoTextStyle}>
              Press shortcut...
            </span>
            <span className="text-[10px] text-[rgba(226,232,240,0.36)]" style={monoTextStyle}>
              Esc cancel
            </span>
            {/* Hidden button to capture key events */}
            <button
              ref={triggerRef}
              type="button"
              className="sr-only"
              onKeyDown={(event) => void handleKeyDown(event)}
              onBlur={() => {
                if (!isSaving) {
                  setIsCapturing(false);
                }
              }}
              aria-label={`Recording shortcut for ${title}`}
            />
          </>
        ) : (
          <>
            {errorMessage ? (
              <span className="text-[11px] text-[#fca5a5]">{errorMessage}</span>
            ) : null}

            {feedbackState === "saved" ? (
              <span className="text-[10px] uppercase tracking-[0.08em] text-[#6ee7b7]" style={monoTextStyle}>
                Saved
              </span>
            ) : null}

            <button
              ref={triggerRef}
              type="button"
              className={cn(
                "group flex items-center gap-2 rounded-md px-1.5 py-1 transition-colors duration-[140ms] ease-out",
                "hover:bg-white/[0.04] disabled:cursor-not-allowed disabled:opacity-50",
              )}
              onClick={startCapture}
              aria-label={`Set shortcut for ${title}`}
              disabled={isDisabled}
            >
              {value ? (
                <ShortcutKeycaps accelerator={value} size="sm" />
              ) : (
                <span className="text-[12px] text-[var(--text-secondary)]" style={monoTextStyle}>
                  Set shortcut
                </span>
              )}
            </button>
          </>
        )}
      </div>
    </div>
  );
}
