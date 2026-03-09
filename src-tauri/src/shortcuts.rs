use std::collections::{HashMap, HashSet};

use log::{info, warn};
use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutEvent, ShortcutState};

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

const DEFAULT_FOCUS_ACCELERATOR: &str = "Ctrl+Shift+Num0";
const DEFAULT_DISPLAY_ACCELERATOR_PREFIX: &str = "Ctrl+Shift+Num";
const MAX_DEFAULT_DISPLAY_BINDINGS: usize = 9;

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

    if let Err(error) = sync_registered_shortcuts(app, state) {
        let mut locked_state = state.lock().expect("state poisoned");
        locked_state.settings = previous_settings.clone();
        drop(locked_state);
        let _ = sync_registered_shortcuts(app, state);
        return Err(error);
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

pub fn sync_registered_shortcuts(app: &AppHandle, state: &SharedState) -> Result<(), String> {
    let planned_shortcuts = build_planned_shortcuts(state)?;

    app.global_shortcut()
        .unregister_all()
        .map_err(|error| format!("Failed to clear existing shortcuts: {error}"))?;

    let mut registered_shortcuts = HashMap::new();

    for planned in &planned_shortcuts {
        app.global_shortcut()
            .register(planned.shortcut)
            .map_err(|error| {
                let _ = app.global_shortcut().unregister_all();
                format!(
                    "Failed to register shortcut {}: {error}",
                    planned.accelerator
                )
            })?;

        registered_shortcuts.insert(planned.shortcut.id(), planned.action.clone());
    }

    let mut locked_state = state.lock().expect("state poisoned");
    locked_state.registered_shortcuts = registered_shortcuts;

    info!(
        "sync_registered_shortcuts: registered {} shortcut(s)",
        planned_shortcuts.len()
    );

    Ok(())
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

    if shortcut.mods.is_empty() {
        return Err(format!(
            "Shortcut {accelerator:?} must include at least one modifier key."
        ));
    }

    Ok(shortcut)
}

fn display_shortcut_label(display: &DisplayState) -> String {
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

fn default_initialized_settings(
    state: &SharedState,
    previous_settings: &AppSettings,
) -> Option<AppSettings> {
    if previous_settings.shortcut_defaults_initialized {
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
        return Some(next_settings);
    }

    let ordered_displays = {
        let state = state.lock().expect("state poisoned");
        let mut displays = state.displays.values().cloned().collect::<Vec<_>>();
        displays.sort_by_key(|display| (display.y, display.x));
        displays
    };

    next_settings.shortcut_settings = build_default_shortcut_settings(&ordered_displays);
    Some(next_settings)
}

fn build_default_shortcut_settings(displays: &[DisplayState]) -> ShortcutSettings {
    let display_bindings = displays
        .iter()
        .take(MAX_DEFAULT_DISPLAY_BINDINGS)
        .enumerate()
        .map(|(index, display)| DisplayHotkeyBinding {
            display_key: display.persistent_key.clone(),
            display_label: format!("Display {}", index + 1),
            accelerator: format!("{DEFAULT_DISPLAY_ACCELERATOR_PREFIX}{}", index + 1),
        })
        .collect();

    ShortcutSettings {
        focus_mode_hotkey: Some(DEFAULT_FOCUS_ACCELERATOR.to_string()),
        display_bindings,
    }
}
