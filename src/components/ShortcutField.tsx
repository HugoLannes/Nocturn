import { useEffect, useRef, useState } from "react";
import { buildAcceleratorFromHeldCodes, isModifierCode } from "../shortcuts";
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
  const heldCodes = useRef(new Set<string>());
  const [isCapturing, setIsCapturing] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [feedbackState, setFeedbackState] = useState<"idle" | "saved">("idle");
  const isSavingRef = useRef(false);

  useEffect(() => {
    isSavingRef.current = isSaving;
  }, [isSaving]);

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

  // Track held keys at the window level so modifier state is always accurate,
  // even when Chromium/WebView2 misreports event.ctrlKey on certain layouts.
  useEffect(() => {
    if (!isCapturing) {
      heldCodes.current.clear();
      return;
    }

    // Delay modifier keyup removal by a short window. Windows sends a
    // synthetic Shift keyup right before numpad keydown when NumLock is on,
    // which would otherwise cause heldCodes to lose Shift before we read it.
    const pendingRemovals = new Map<string, number>();

    function onKeyDown(event: KeyboardEvent) {
      const pending = pendingRemovals.get(event.code);
      if (pending !== undefined) {
        window.clearTimeout(pending);
        pendingRemovals.delete(event.code);
      }
      heldCodes.current.add(event.code);
    }

    function onKeyUp(event: KeyboardEvent) {
      if (isModifierCode(event.code)) {
        const timer = window.setTimeout(() => {
          heldCodes.current.delete(event.code);
          pendingRemovals.delete(event.code);
        }, 50);
        pendingRemovals.set(event.code, timer);
      } else {
        heldCodes.current.delete(event.code);
      }
    }

    function onWindowBlur() {
      for (const timer of pendingRemovals.values()) window.clearTimeout(timer);
      pendingRemovals.clear();
      heldCodes.current.clear();
    }

    window.addEventListener("keydown", onKeyDown, true);
    window.addEventListener("keyup", onKeyUp, true);
    window.addEventListener("blur", onWindowBlur);

    return () => {
      window.removeEventListener("keydown", onKeyDown, true);
      window.removeEventListener("keyup", onKeyUp, true);
      window.removeEventListener("blur", onWindowBlur);
      for (const timer of pendingRemovals.values()) window.clearTimeout(timer);
      pendingRemovals.clear();
      heldCodes.current.clear();
    };
  }, [isCapturing]);

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

  function handleKeyDown(event: React.KeyboardEvent<HTMLButtonElement>) {
    if (!isCapturing || disabled || isSavingRef.current) {
      return;
    }

    event.preventDefault();
    event.stopPropagation();

    if (event.key === "Escape") {
      setIsCapturing(false);
      setErrorMessage(null);
      return;
    }

    // Allow bare Backspace/Delete to clear the shortcut (check held codes
    // for modifiers instead of event flags, consistent with the rest).
    if (
      (event.key === "Backspace" || event.key === "Delete")
      && !heldCodes.current.has("AltLeft") && !heldCodes.current.has("AltRight")
      && !heldCodes.current.has("ControlLeft") && !heldCodes.current.has("ControlRight")
      && !heldCodes.current.has("MetaLeft") && !heldCodes.current.has("MetaRight")
      && !heldCodes.current.has("ShiftLeft") && !heldCodes.current.has("ShiftRight")
    ) {
      void submitShortcut(null);
      return;
    }

    if (isModifierCode(event.nativeEvent.code)) {
      return;
    }

    // Build accelerator by merging manually tracked modifier state with
    // event flags — covers both AZERTY and NumLock+Shift edge cases.
    const accelerator = buildAcceleratorFromHeldCodes(heldCodes.current, event.nativeEvent);
    if (!accelerator) {
      setErrorMessage("Use at least one modifier key.");
      return;
    }

    void submitShortcut(accelerator);
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
              onKeyDown={handleKeyDown}
              onBlur={() => {
                if (!isSavingRef.current) {
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
