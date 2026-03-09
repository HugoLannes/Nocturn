import { useState, useRef, useEffect, type ReactNode } from "react";
import type { Display, ShortcutSettings, ShortcutSettingsInput } from "../types";
import { cn, layoutEyebrowClass, layoutTitleClass, monoTextStyle } from "../ui";
import { ShortcutField } from "./ShortcutField";

type SettingsPageProps = {
  displays: Display[];
  shortcutSettings: ShortcutSettings;
  allowCursorExitActiveDisplays: boolean;
  showOverlayHiddenApps: boolean;
  isMutating: boolean;
  onUpdateShortcutSettings: (settings: ShortcutSettingsInput) => Promise<string | null>;
  onToggleAllowCursorExitActiveDisplays: (allowed: boolean) => void;
  onToggleShowOverlayHiddenApps: (enabled: boolean) => void;
  onResetToDefaults: () => void;
};

type SettingsSectionProps = {
  eyebrow: string;
  title: string;
  children: ReactNode;
};

type SettingsToggleRowProps = {
  title: string;
  hint: string;
  ariaLabel: string;
  enabled: boolean;
  disabled: boolean;
  onToggle: () => void;
};

const sectionClass = "rounded-[20px] border border-[var(--border)] px-5 py-4 shadow-[inset_0_1px_0_rgba(255,255,255,0.04),0_18px_40px_rgba(2,6,23,0.16)] [background:linear-gradient(180deg,rgba(255,255,255,0.036),rgba(255,255,255,0.018)_42%,rgba(255,255,255,0.022))] max-[560px]:px-4 max-[560px]:py-3.5";
const rowClass = "flex items-center justify-between gap-4 py-2.5 max-[560px]:py-2";
const dividerClass = "border-t border-white/[0.065]";
const groupLabelClass = "inline-block pt-2.5 pb-1 text-[11px] uppercase tracking-[0.08em] text-[rgba(226,232,240,0.44)]";

function displayShortcutTitle(display: Display, index: number) {
  const match = display.name.match(/DISPLAY(\d+)/i);
  if (match) {
    return `Display ${match[1]}`;
  }

  return display.isPrimary ? "Primary display" : `Display ${index + 1}`;
}

function displayShortcutHint(display: Display) {
  const descriptor = [display.manufacturer, display.model].filter(Boolean).join(" ");
  const meta = `${display.width}x${display.height}`;
  const details = descriptor ? `${descriptor} · ${meta}` : meta;
  return display.isPrimary ? `Primary · ${details}` : details;
}

function toShortcutSettingsInput(shortcutSettings: ShortcutSettings): ShortcutSettingsInput {
  return {
    focusModeHotkey: shortcutSettings.focusModeHotkey,
    displayBindings: shortcutSettings.displayBindings.map(({ displayKey, displayLabel, accelerator }) => ({
      displayKey,
      displayLabel,
      accelerator,
    })),
  };
}

function SettingsSection({ eyebrow, title, children }: SettingsSectionProps) {
  return (
    <article className={sectionClass}>
      <div className="flex items-baseline gap-2 pb-2">
        <span className={layoutEyebrowClass} style={monoTextStyle}>{eyebrow}</span>
        <h2 className="text-[15px] font-semibold leading-none tracking-[-0.03em] text-[var(--accent-soft)] max-[560px]:text-[14px]">
          {title}
        </h2>
      </div>
      <div className={dividerClass}>{children}</div>
    </article>
  );
}

function SettingsToggleRow({
  title,
  hint,
  ariaLabel,
  enabled,
  disabled,
  onToggle,
}: SettingsToggleRowProps) {
  return (
    <div className={rowClass}>
      <div className="min-w-0">
        <span className="text-[14px] font-semibold leading-[1.15] tracking-[-0.02em] text-[var(--text-primary)]">{title}</span>
        <p className="mt-0.5 text-[12px] leading-[1.5] text-[rgba(226,232,240,0.56)]">{hint}</p>
      </div>

      <button
        type="button"
        role="switch"
        className={cn(
          "relative h-5 w-9 shrink-0 rounded-full border transition-[background,border-color,box-shadow] duration-[140ms] ease-out disabled:cursor-not-allowed disabled:opacity-50",
          enabled
            ? "border-[rgba(var(--accent-rgb),0.34)] bg-[linear-gradient(90deg,rgba(var(--accent-rgb),0.94),rgba(var(--accent-rgb),0.62))] shadow-[inset_0_1px_0_rgba(255,255,255,0.12),0_4px_12px_rgba(var(--accent-rgb),0.22)]"
            : "border-white/10 bg-[rgba(148,163,184,0.12)] shadow-[inset_0_1px_0_rgba(255,255,255,0.04)] hover:bg-[rgba(148,163,184,0.20)]",
        )}
        onClick={onToggle}
        aria-label={ariaLabel}
        aria-checked={enabled}
        disabled={disabled}
      >
        <span
          className={cn(
            "block h-3.5 w-3.5 rounded-full transition-transform duration-[140ms] ease-out",
            enabled
              ? "translate-x-[17px] bg-white shadow-[0_1px_4px_rgba(15,23,42,0.3)]"
              : "translate-x-[3px] bg-[rgba(148,163,184,0.55)] shadow-[0_1px_3px_rgba(15,23,42,0.2)]",
          )}
        />
      </button>
    </div>
  );
}

export function SettingsPage({
  displays,
  shortcutSettings,
  allowCursorExitActiveDisplays,
  showOverlayHiddenApps,
  isMutating,
  onUpdateShortcutSettings,
  onToggleAllowCursorExitActiveDisplays,
  onToggleShowOverlayHiddenApps,
  onResetToDefaults,
}: SettingsPageProps) {
  const unavailableBindings = shortcutSettings.displayBindings.filter((binding) => !binding.isAvailable);
  const [isConfirming, setIsConfirming] = useState(false);
  const confirmTimerRef = useRef<ReturnType<typeof setTimeout>>(undefined);

  useEffect(() => {
    return () => {
      if (confirmTimerRef.current) clearTimeout(confirmTimerRef.current);
    };
  }, []);

  function handleResetClick() {
    if (!isConfirming) {
      setIsConfirming(true);
      confirmTimerRef.current = setTimeout(() => setIsConfirming(false), 3000);
      return;
    }

    if (confirmTimerRef.current) clearTimeout(confirmTimerRef.current);
    setIsConfirming(false);
    onResetToDefaults();
  }

  async function updateShortcutSettings(mutator: (draft: ShortcutSettingsInput) => void) {
    const nextSettings = toShortcutSettingsInput(shortcutSettings);
    mutator(nextSettings);
    return onUpdateShortcutSettings(nextSettings);
  }

  return (
    <section className="flex h-full min-h-0 flex-col gap-3" aria-label="Settings">
      <header className="flex shrink-0 items-end justify-between px-[2px] pt-[2px]">
        <div>
          <span className={layoutEyebrowClass} style={monoTextStyle}>Settings</span>
          <h1 className={layoutTitleClass}>Set things your way.</h1>
        </div>
        <button
          type="button"
          className={cn(
            "mb-[2px] rounded-[8px] border px-2.5 py-[4px] text-[11px] font-medium tracking-[-0.01em] transition-all duration-[140ms] ease-out disabled:cursor-not-allowed disabled:opacity-50",
            isConfirming
              ? "border-red-500/30 bg-red-500/10 text-red-400 hover:border-red-500/50 hover:bg-red-500/20"
              : "border-white/8 bg-white/[0.03] text-[rgba(226,232,240,0.48)] hover:border-white/12 hover:bg-white/[0.06] hover:text-[rgba(226,232,240,0.68)]",
          )}
          disabled={isMutating}
          onClick={handleResetClick}
        >
          {isConfirming ? "Confirm" : "Reset to defaults"}
        </button>
      </header>

      <div className="nocturn-scrollbar min-h-0 flex-1 overflow-y-auto px-[2px] pb-[6px] pr-1.5 pt-[2px] max-[560px]:pr-0.5">
        <div className="flex flex-col gap-3">
          <SettingsSection eyebrow="General" title="Display and cursor">
            <SettingsToggleRow
              title="Show hidden apps on overlays"
              hint="Display apps opened behind blackout overlays."
              ariaLabel="Show hidden apps on blackout overlays"
              enabled={showOverlayHiddenApps}
              disabled={isMutating}
              onToggle={() => onToggleShowOverlayHiddenApps(!showOverlayHiddenApps)}
            />
            <div className={dividerClass}>
              <SettingsToggleRow
                title="Free cursor movement"
                hint="Let the pointer travel outside active monitors."
                ariaLabel="Allow mouse to leave active displays"
                enabled={allowCursorExitActiveDisplays}
                disabled={isMutating}
                onToggle={() => onToggleAllowCursorExitActiveDisplays(!allowCursorExitActiveDisplays)}
              />
            </div>
          </SettingsSection>

          <SettingsSection eyebrow="Keyboard control" title="Hotkeys">
            <div className="py-1">
              <ShortcutField
                title="Focus mode"
                hint="Primary only"
                value={shortcutSettings.focusModeHotkey}
                disabled={isMutating}
                onSubmit={(accelerator) => updateShortcutSettings((draft) => {
                  draft.focusModeHotkey = accelerator;
                })}
              />
            </div>

            {displays.length > 0 ? (
              <div className={dividerClass}>
                <span className={groupLabelClass} style={monoTextStyle}>
                  Connected displays
                </span>
                {displays.map((display, index) => (
                  <ShortcutField
                    key={display.persistentKey}
                    title={displayShortcutTitle(display, index)}
                    hint={displayShortcutHint(display)}
                    value={display.hotkey}
                    disabled={isMutating}
                    onSubmit={(accelerator) => updateShortcutSettings((draft) => {
                      draft.displayBindings = draft.displayBindings.filter(
                        (binding) => binding.displayKey !== display.persistentKey,
                      );

                      if (accelerator) {
                        draft.displayBindings.push({
                          displayKey: display.persistentKey,
                          displayLabel: displayShortcutTitle(display, index),
                          accelerator,
                        });
                      }
                    })}
                  />
                ))}
              </div>
            ) : null}

            {unavailableBindings.length > 0 ? (
              <div className={dividerClass}>
                <span className={cn(groupLabelClass, "text-[rgba(226,232,240,0.38)]")} style={monoTextStyle}>
                  Unavailable
                </span>
                {unavailableBindings.map((binding) => (
                  <ShortcutField
                    key={binding.displayKey}
                    title={binding.displayLabel}
                    hint="Disconnected"
                    value={binding.accelerator}
                    disabled={isMutating}
                    statusText="Not available"
                    onSubmit={(accelerator) => updateShortcutSettings((draft) => {
                      draft.displayBindings = draft.displayBindings.filter(
                        (item) => item.displayKey !== binding.displayKey,
                      );

                      if (accelerator) {
                        draft.displayBindings.push({
                          displayKey: binding.displayKey,
                          displayLabel: binding.displayLabel,
                          accelerator,
                        });
                      }
                    })}
                  />
                ))}
              </div>
            ) : null}
          </SettingsSection>
        </div>
      </div>
    </section>
  );
}
