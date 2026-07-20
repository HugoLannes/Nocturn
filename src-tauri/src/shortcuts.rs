use std::collections::{HashMap, HashSet};

use log::{info, warn};
use tauri::AppHandle;
use tauri_plugin_global_shortcut::{
    Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutEvent, ShortcutState,
};

use crate::{
    commands::{self, SharedState},
    settings::{self, AppSettings, DisplayHotkeyBinding, ShortcutSettings},
    state::{DisplayState, ShortcutAction},
};

struct PlannedShortcut {
    accelerator: String,
    shortcut: Shortcut,
    action: ShortcutAction,
}

// Top-row digit keys are used (rather than the numpad) because Windows only
// generates VK_NUMPADx while NumLock is on, which would make the registered
// hotkey silently unreachable whenever NumLock is off, and laptops/TKL
// keyboards without a numpad could never trigger it at all.
const DEFAULT_FOCUS_ACCELERATOR: &str = "Ctrl+Shift+Digit0";
const DEFAULT_DISPLAY_ACCELERATOR_PREFIX: &str = "Ctrl+Shift+Digit";
// Previous defaults, kept only to detect and migrate untouched installs.
const LEGACY_NUMPAD_FOCUS_ACCELERATOR: &str = "Ctrl+Shift+Num0";
const LEGACY_NUMPAD_DISPLAY_ACCELERATOR_PREFIX: &str = "Ctrl+Shift+Num";
const MAX_DEFAULT_DISPLAY_BINDINGS: usize = 9;

fn validate_shortcut_policy(shortcut: &Shortcut) -> Result<(), String> {
    if shortcut.mods.intersects(Modifiers::SUPER | Modifiers::META) {
        return Err("Windows key shortcuts are reserved by the OS and cannot be used.".to_string());
    }

    match shortcut.key {
        Code::F12 => Err(
            "F12 is reserved by Windows for debugging and cannot be used as a global shortcut."
                .to_string(),
        ),
        Code::F4 if shortcut.mods.contains(Modifiers::ALT) => Err(
            "Alt+F4 closes the active window and cannot be used as a global shortcut."
                .to_string(),
        ),
        Code::Tab if shortcut.mods.contains(Modifiers::ALT) => Err(
            "Alt+Tab is reserved by Windows task switching and cannot be used as a global shortcut."
                .to_string(),
        ),
        Code::Space if shortcut.mods.contains(Modifiers::ALT) => Err(
            "Alt+Space opens the system window menu and cannot be used as a global shortcut."
                .to_string(),
        ),
        Code::Escape if shortcut.mods.contains(Modifiers::ALT) => Err(
            "Alt+Esc is reserved by Windows window switching and cannot be used as a global shortcut."
                .to_string(),
        ),
        Code::Escape if shortcut.mods.contains(Modifiers::CONTROL) => Err(
            "Ctrl+Esc and Ctrl+Shift+Esc are reserved by Windows and cannot be used as global shortcuts."
                .to_string(),
        ),
        Code::Delete
            if shortcut.mods.contains(Modifiers::CONTROL)
                && shortcut.mods.contains(Modifiers::ALT) =>
        {
            Err(
                "Ctrl+Alt+Delete is reserved by Windows security and cannot be used as a global shortcut."
                    .to_string(),
            )
        }
        _ => Ok(()),
    }
}

fn describe_registration_error(accelerator: &str, error: impl std::fmt::Display) -> String {
    let error_text = error.to_string();
    let normalized = error_text.to_ascii_lowercase();

    if normalized.contains("already registered") {
        return format!(
            "Shortcut {accelerator} is already used by another application or by Windows."
        );
    }

    if normalized.contains("unknown vkcode") {
        return format!(
            "Shortcut {accelerator} is not supported by the current Windows global shortcut backend."
        );
    }

    format!("Failed to register shortcut {accelerator}: {error_text}")
}

pub fn ensure_default_shortcuts(app: &AppHandle, state: &SharedState) -> Result<bool, String> {
    let previous_settings = {
        let state = state.lock().expect("state poisoned");
        state.settings.clone()
    };

    let Some(next_settings) = default_initialized_settings(state, &previous_settings) else {
        return Ok(false);
    };

    {
        let mut state = state.lock().expect("state poisoned");
        state.settings = next_settings.clone();
    }

    match sync_registered_shortcuts(app, state) {
        Ok(failures) => {
            for failure in &failures {
                warn!("ensure_default_shortcuts: {failure}");
            }
        }
        Err(error) => {
            let mut locked_state = state.lock().expect("state poisoned");
            locked_state.settings = previous_settings.clone();
            drop(locked_state);
            let _ = sync_registered_shortcuts(app, state);
            return Err(error);
        }
    }

    if let Err(error) = settings::save_settings(app, &next_settings) {
        let mut locked_state = state.lock().expect("state poisoned");
        locked_state.settings = previous_settings;
        drop(locked_state);
        let _ = sync_registered_shortcuts(app, state);
        return Err(error);
    }

    Ok(true)
}

pub fn sanitize_shortcut_settings(
    state: &SharedState,
    shortcut_settings: ShortcutSettings,
) -> Result<ShortcutSettings, String> {
    let display_labels_by_key = {
        let state = state.lock().expect("state poisoned");
        state
            .displays
            .values()
            .map(|display| {
                (
                    display.persistent_key.clone(),
                    display_shortcut_label(display),
                )
            })
            .collect::<HashMap<_, _>>()
    };

    let mut seen_shortcuts = HashMap::<u32, String>::new();
    let mut seen_display_keys = HashSet::<String>::new();

    let focus_mode_hotkey = shortcut_settings
        .focus_mode_hotkey
        .as_deref()
        .map(parse_shortcut)
        .transpose()?
        .map(|shortcut| {
            seen_shortcuts.insert(shortcut.id(), "Focus mode".to_string());
            shortcut.to_string()
        });

    let mut display_bindings = Vec::new();

    for binding in shortcut_settings.display_bindings {
        let display_key = binding.display_key.trim();
        if display_key.is_empty() {
            continue;
        }

        if !seen_display_keys.insert(display_key.to_string()) {
            return Err(format!(
                "Only one shortcut can be stored for display {}.",
                binding.display_label.trim()
            ));
        }

        let accelerator = binding.accelerator.trim();
        if accelerator.is_empty() {
            continue;
        }

        let shortcut = parse_shortcut(accelerator)?;
        let display_label = display_labels_by_key
            .get(display_key)
            .cloned()
            .or_else(|| {
                let label = binding.display_label.trim();
                if label.is_empty() {
                    None
                } else {
                    Some(label.to_string())
                }
            })
            .unwrap_or_else(|| display_key.to_string());

        if let Some(existing_owner) = seen_shortcuts.insert(shortcut.id(), display_label.clone()) {
            return Err(format!(
                "Shortcut {} is already used by {}.",
                accelerator, existing_owner
            ));
        }

        display_bindings.push(DisplayHotkeyBinding {
            display_key: display_key.to_string(),
            display_label,
            accelerator: shortcut.to_string(),
        });
    }

    Ok(ShortcutSettings {
        focus_mode_hotkey,
        display_bindings,
    })
}

/// (Re)registers every planned shortcut with the OS.
///
/// Registration is best-effort per shortcut: a single accelerator already
/// owned by another application (or a second Nocturn instance) no longer
/// takes down every other hotkey. Shortcuts that fail to register are
/// skipped and their description is returned in the `Vec`; `Err` is reserved
/// for failures that make the whole sync impossible (e.g. `unregister_all`
/// itself failing, or a stored accelerator that can no longer be parsed).
pub fn sync_registered_shortcuts(app: &AppHandle, state: &SharedState) -> Result<Vec<String>, String> {
    let planned_shortcuts = build_planned_shortcuts(state)?;

    app.global_shortcut()
        .unregister_all()
        .map_err(|error| format!("Failed to clear existing shortcuts: {error}"))?;

    let mut registered_shortcuts = HashMap::new();
    let mut registration_failures = Vec::new();

    for planned in &planned_shortcuts {
        if let Err(error) = app.global_shortcut().register(planned.shortcut) {
            let message = describe_registration_error(&planned.accelerator, error);
            warn!("sync_registered_shortcuts: {message}");
            registration_failures.push(message);
            continue;
        }

        registered_shortcuts.insert(planned.shortcut.id(), planned.action.clone());
    }

    let mut locked_state = state.lock().expect("state poisoned");
    locked_state.registered_shortcuts = registered_shortcuts;
    drop(locked_state);

    info!(
        "sync_registered_shortcuts: registered {} of {} shortcut(s)",
        planned_shortcuts.len() - registration_failures.len(),
        planned_shortcuts.len()
    );

    Ok(registration_failures)
}

pub fn handle_shortcut_event(
    app: &AppHandle,
    shortcut: &Shortcut,
    event: ShortcutEvent,
    state: &SharedState,
) {
    if event.state() != ShortcutState::Pressed {
        return;
    }

    let Some(action) = ({
        let state = state.lock().expect("state poisoned");
        state.registered_shortcuts.get(&shortcut.id()).cloned()
    }) else {
        return;
    };

    if let Err(error) = commands::execute_shortcut_action(app, state, action) {
        warn!(
            "handle_shortcut_event: failed to run shortcut {}: {}",
            shortcut, error
        );
    }
}

fn build_planned_shortcuts(state: &SharedState) -> Result<Vec<PlannedShortcut>, String> {
    let (shortcut_settings, available_display_keys) = {
        let state = state.lock().expect("state poisoned");
        (
            state.settings.shortcut_settings.clone(),
            state
                .displays
                .values()
                .map(|display| display.persistent_key.clone())
                .collect::<HashSet<_>>(),
        )
    };

    let mut planned_shortcuts = Vec::new();

    if let Some(accelerator) = shortcut_settings.focus_mode_hotkey {
        let shortcut = parse_shortcut(&accelerator)?;
        planned_shortcuts.push(PlannedShortcut {
            accelerator,
            shortcut,
            action: ShortcutAction::FocusMode,
        });
    }

    for binding in shortcut_settings.display_bindings {
        if !available_display_keys.contains(&binding.display_key) {
            continue;
        }

        let shortcut = parse_shortcut(&binding.accelerator)?;
        planned_shortcuts.push(PlannedShortcut {
            accelerator: binding.accelerator,
            shortcut,
            action: ShortcutAction::ToggleDisplay {
                display_key: binding.display_key,
            },
        });
    }

    Ok(planned_shortcuts)
}

fn parse_shortcut(raw: &str) -> Result<Shortcut, String> {
    let accelerator = raw.trim();
    let shortcut = accelerator
        .parse::<Shortcut>()
        .map_err(|error| format!("Invalid shortcut {accelerator:?}: {error}"))?;

    validate_shortcut_policy(&shortcut)?;

    Ok(shortcut)
}

fn display_shortcut_label(display: &DisplayState) -> String {
    if let Some(number) = display_number(display) {
        return format!("Display {number}");
    }

    if display.is_primary {
        return "Primary display".to_string();
    }

    let descriptor = [display.manufacturer.as_str(), display.model.as_str()]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    if !descriptor.is_empty() {
        descriptor
    } else if !display.name.is_empty() {
        display.name.clone()
    } else {
        "Display".to_string()
    }
}

fn display_number(display: &DisplayState) -> Option<usize> {
    let upper_name = display.name.to_ascii_uppercase();
    let marker_index = upper_name.find("DISPLAY")? + "DISPLAY".len();
    let digits = upper_name[marker_index..]
        .chars()
        .take_while(|character| character.is_ascii_digit())
        .collect::<String>();

    if digits.is_empty() {
        return None;
    }

    digits.parse::<usize>().ok()
}

fn next_available_default_slot(used_slots: &HashSet<usize>) -> Option<usize> {
    (1..=MAX_DEFAULT_DISPLAY_BINDINGS).find(|slot| !used_slots.contains(slot))
}

fn default_initialized_settings(
    state: &SharedState,
    previous_settings: &AppSettings,
) -> Option<AppSettings> {
    let ordered_displays = {
        let state = state.lock().expect("state poisoned");
        let mut displays = state.displays.values().cloned().collect::<Vec<_>>();
        displays.sort_by_key(|display| (display.y, display.x));
        displays
    };
    let legacy_shortcut_settings = build_legacy_default_shortcut_settings(&ordered_displays);
    let next_shortcut_settings = build_default_shortcut_settings(&ordered_displays);

    if previous_settings.shortcut_defaults_initialized {
        if previous_settings.shortcut_settings == legacy_shortcut_settings
            && previous_settings.shortcut_settings != next_shortcut_settings
        {
            let mut next_settings = previous_settings.clone();
            next_settings.shortcut_settings = next_shortcut_settings;
            return Some(next_settings);
        }

        // Untouched installs from before the Numpad -> top-row-digit default
        // change: migrate them so the out-of-box hotkeys actually fire
        // regardless of NumLock state or numpad availability.
        let legacy_numpad_shortcut_settings = build_legacy_default_shortcut_settings_with_accelerators(
            &ordered_displays,
            LEGACY_NUMPAD_FOCUS_ACCELERATOR,
            LEGACY_NUMPAD_DISPLAY_ACCELERATOR_PREFIX,
        );
        let next_numpad_shortcut_settings = build_default_shortcut_settings_with_accelerators(
            &ordered_displays,
            LEGACY_NUMPAD_FOCUS_ACCELERATOR,
            LEGACY_NUMPAD_DISPLAY_ACCELERATOR_PREFIX,
        );

        if previous_settings.shortcut_settings == legacy_numpad_shortcut_settings
            || previous_settings.shortcut_settings == next_numpad_shortcut_settings
        {
            let mut next_settings = previous_settings.clone();
            next_settings.shortcut_settings = next_shortcut_settings;
            return Some(next_settings);
        }

        return None;
    }

    let mut next_settings = previous_settings.clone();
    next_settings.shortcut_defaults_initialized = true;

    if previous_settings
        .shortcut_settings
        .focus_mode_hotkey
        .is_some()
        || !previous_settings
            .shortcut_settings
            .display_bindings
            .is_empty()
    {
        if previous_settings.shortcut_settings == legacy_shortcut_settings
            && previous_settings.shortcut_settings != next_shortcut_settings
        {
            next_settings.shortcut_settings = next_shortcut_settings;
        }

        return Some(next_settings);
    }

    next_settings.shortcut_settings = next_shortcut_settings;
    Some(next_settings)
}

fn build_default_shortcut_settings(displays: &[DisplayState]) -> ShortcutSettings {
    build_default_shortcut_settings_with_accelerators(
        displays,
        DEFAULT_FOCUS_ACCELERATOR,
        DEFAULT_DISPLAY_ACCELERATOR_PREFIX,
    )
}

fn build_default_shortcut_settings_with_accelerators(
    displays: &[DisplayState],
    focus_accelerator: &str,
    display_accelerator_prefix: &str,
) -> ShortcutSettings {
    let display_bindings = displays
        .iter()
        .scan(HashSet::<usize>::new(), |used_slots, display| {
            let slot = display_number(display)
                .filter(|slot| (1..=MAX_DEFAULT_DISPLAY_BINDINGS).contains(slot))
                .filter(|slot| used_slots.insert(*slot))
                .or_else(|| {
                    let slot = next_available_default_slot(used_slots)?;
                    used_slots.insert(slot);
                    Some(slot)
                });

            Some(slot.map(|slot| DisplayHotkeyBinding {
                display_key: display.persistent_key.clone(),
                display_label: display_shortcut_label(display),
                accelerator: format!("{display_accelerator_prefix}{slot}"),
            }))
        })
        .flatten()
        .collect();

    ShortcutSettings {
        focus_mode_hotkey: Some(focus_accelerator.to_string()),
        display_bindings,
    }
}

fn build_legacy_default_shortcut_settings(displays: &[DisplayState]) -> ShortcutSettings {
    build_legacy_default_shortcut_settings_with_accelerators(
        displays,
        DEFAULT_FOCUS_ACCELERATOR,
        DEFAULT_DISPLAY_ACCELERATOR_PREFIX,
    )
}

fn build_legacy_default_shortcut_settings_with_accelerators(
    displays: &[DisplayState],
    focus_accelerator: &str,
    display_accelerator_prefix: &str,
) -> ShortcutSettings {
    let display_bindings = displays
        .iter()
        .take(MAX_DEFAULT_DISPLAY_BINDINGS)
        .enumerate()
        .map(|(index, display)| DisplayHotkeyBinding {
            display_key: display.persistent_key.clone(),
            display_label: format!("Display {}", index + 1),
            accelerator: format!("{display_accelerator_prefix}{}", index + 1),
        })
        .collect();

    ShortcutSettings {
        focus_mode_hotkey: Some(focus_accelerator.to_string()),
        display_bindings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_non_reserved_single_key_shortcuts() {
        assert!(parse_shortcut("KeyA").is_ok());
        assert!(parse_shortcut("F11").is_ok());
        assert!(parse_shortcut("Ctrl+Shift+Numpad0").is_ok());
    }

    #[test]
    fn default_accelerators_are_top_row_digits_not_numpad() {
        // Default hotkeys must not depend on NumLock or a physical numpad.
        assert!(DEFAULT_FOCUS_ACCELERATOR.contains("Digit"));
        assert!(DEFAULT_DISPLAY_ACCELERATOR_PREFIX.contains("Digit"));
        assert!(parse_shortcut(DEFAULT_FOCUS_ACCELERATOR).is_ok());
        for slot in 1..=MAX_DEFAULT_DISPLAY_BINDINGS {
            assert!(parse_shortcut(&format!("{DEFAULT_DISPLAY_ACCELERATOR_PREFIX}{slot}")).is_ok());
        }
    }

    #[test]
    fn rejects_reserved_windows_shortcuts() {
        assert!(parse_shortcut("F12").unwrap_err().contains("F12"));
        assert!(parse_shortcut("Alt+F4").unwrap_err().contains("Alt+F4"));
        assert!(parse_shortcut("Alt+Tab").unwrap_err().contains("Alt+Tab"));
        assert!(parse_shortcut("Ctrl+Alt+Delete")
            .unwrap_err()
            .contains("Ctrl+Alt+Delete"));
        assert!(parse_shortcut("Super+KeyN")
            .unwrap_err()
            .contains("Windows key"));
    }
}
