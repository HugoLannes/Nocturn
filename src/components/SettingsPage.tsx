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
};

type SettingsToggleCardProps = {
  title: string;
  label: string;
  hint: string;
  ariaLabel: string;
  enabled: boolean;
  disabled: boolean;
  onToggle: () => void;
};

const settingsCardClass = "rounded-[18px] border border-[var(--border)] px-[18px] pb-4 pt-[18px] max-[560px]:p-4 [background:linear-gradient(180deg,rgba(255,255,255,0.04),rgba(255,255,255,0.015)),rgba(255,255,255,0.02)]";
const settingsRowClass = "flex items-center justify-between gap-[22px] border-t border-white/6 pt-4 max-[560px]:flex-col max-[560px]:items-stretch max-[560px]:gap-[14px] max-[560px]:pt-[14px]";

function displayShortcutTitle(display: Display, index: number) {
  return display.isPrimary ? "Primary display" : `Display ${index + 1}`;
}

function displayShortcutHint(display: Display) {
  const descriptor = [display.manufacturer, display.model].filter(Boolean).join(" ");
  const meta = `${display.width}x${display.height}`;
  return descriptor ? `${descriptor} · ${meta}` : meta;
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

function SettingsToggleCard({
  title,
  label,
  hint,
  ariaLabel,
  enabled,
  disabled,
  onToggle,
}: SettingsToggleCardProps) {
  return (
    <article className={settingsCardClass}>
      <div className="flex items-center justify-between gap-[22px] pb-4 max-[560px]:flex-col max-[560px]:items-stretch max-[560px]:gap-3 max-[560px]:pb-[14px]">
        <div>
          <h2 className="text-[22px] font-semibold leading-none tracking-[0.02em] text-[var(--accent-soft)]">{title}</h2>
        </div>
      </div>

      <label className={settingsRowClass}>
        <div>
          <span className="block text-[14px] font-semibold leading-[1.15]">{label}</span>
          <p className="mt-1.5 max-w-[560px] text-[13px] leading-[1.6] text-[rgba(226,232,240,0.66)]">
            {hint}
          </p>
        </div>

        <button
          type="button"
          className={cn(
            "flex h-8 w-[58px] shrink-0 items-center rounded-full border-0 p-1 transition-[background,box-shadow] duration-[140ms] ease-out disabled:cursor-not-allowed disabled:opacity-50 max-[560px]:w-[52px]",
            enabled
              ? "bg-[linear-gradient(90deg,rgba(var(--accent-rgb),0.94),rgba(var(--accent-rgb),0.76))] shadow-[inset_0_0_0_1px_rgba(var(--accent-rgb),0.24),0_4px_18px_rgba(var(--accent-rgb),0.28)] hover:shadow-[inset_0_0_0_1px_rgba(var(--accent-rgb),0.24),0_4px_18px_rgba(var(--accent-rgb),0.34)]"
              : "bg-[rgba(148,163,184,0.16)] shadow-[inset_0_0_0_1px_rgba(255,255,255,0.08)] hover:bg-[rgba(148,163,184,0.22)]",
          )}
          onClick={onToggle}
          aria-label={ariaLabel}
          aria-pressed={enabled}
          disabled={disabled}
        >
          <span
            className={cn(
              "h-[22px] w-[22px] rounded-full bg-[#e2e8f0] shadow-[0_4px_12px_rgba(15,23,42,0.25)] transition-transform duration-[140ms] ease-out",
              enabled && "translate-x-[26px]",
            )}
          />
        </button>
      </label>
    </article>
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
}: SettingsPageProps) {
  const unavailableBindings = shortcutSettings.displayBindings.filter((binding) => !binding.isAvailable);

  async function updateShortcutSettings(mutator: (draft: ShortcutSettingsInput) => void) {
    const nextSettings = toShortcutSettingsInput(shortcutSettings);
    mutator(nextSettings);
    return onUpdateShortcutSettings(nextSettings);
  }

  return (
    <section className="flex h-full min-h-0 flex-col gap-3" aria-label="Settings">
      <header className="flex flex-col gap-[3px] px-[2px] pt-[2px] max-[560px]:items-stretch">
        <span className={layoutEyebrowClass} style={monoTextStyle}>Settings</span>
        <h1 className={layoutTitleClass}>Set things your way.</h1>
      </header>

      <div className="flex min-h-0 flex-1 flex-col gap-3 overflow-auto px-[2px] pb-[6px] pt-[2px] max-[560px]:gap-[10px] max-[560px]:pr-0">
        <SettingsToggleCard
          title="Overlays"
          label="Show hidden apps"
          hint="Display the app names detected behind blackout overlays. Turn this off for a fully plain black screen."
          ariaLabel="Show hidden apps on blackout overlays"
          enabled={showOverlayHiddenApps}
          disabled={isMutating}
          onToggle={() => onToggleShowOverlayHiddenApps(!showOverlayHiddenApps)}
        />

        <SettingsToggleCard
          title="Cursor"
          label="Cursor freedom"
          hint="Turn this on to let the mouse travel outside the active monitors. Turn it off to confine the pointer."
          ariaLabel="Allow mouse to leave active displays"
          enabled={allowCursorExitActiveDisplays}
          disabled={isMutating}
          onToggle={() => onToggleAllowCursorExitActiveDisplays(!allowCursorExitActiveDisplays)}
        />

        <article className={settingsCardClass}>
          <div className="flex flex-col gap-1 pb-4 max-[560px]:pb-[14px]">
            <h2 className="text-[22px] font-semibold leading-none tracking-[0.02em] text-[var(--accent-soft)]">Hotkeys</h2>
            <p className="max-w-[620px] text-[13px] leading-[1.6] text-[rgba(226,232,240,0.66)]">
              Set global shortcuts for Focus mode and each display. They keep working while Nocturn is minimized.
            </p>
          </div>

          <div className="flex flex-col gap-3 border-t border-white/6 pt-4 max-[560px]:pt-[14px]">
            <ShortcutField
              title="Focus mode"
              hint="Keep only the primary display active."
              value={shortcutSettings.focusModeHotkey}
              disabled={isMutating}
              onSubmit={(accelerator) => updateShortcutSettings((draft) => {
                draft.focusModeHotkey = accelerator;
              })}
            />

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

            {unavailableBindings.length > 0 ? (
              <div className="flex flex-col gap-3 pt-1">
                <span className={layoutEyebrowClass} style={monoTextStyle}>Unavailable displays</span>
                {unavailableBindings.map((binding) => (
                  <ShortcutField
                    key={binding.displayKey}
                    title={binding.displayLabel}
                    hint="This display is not currently connected. The shortcut is kept until you clear it."
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
          </div>
        </article>
      </div>
    </section>
  );
}
