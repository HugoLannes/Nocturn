use std::{fs, path::PathBuf};

use log::{error, warn};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct DisplayHotkeyBinding {
    pub display_key: String,
    pub display_label: String,
    pub accelerator: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct ShortcutSettings {
    pub focus_mode_hotkey: Option<String>,
    pub display_bindings: Vec<DisplayHotkeyBinding>,
}

/// Bumped whenever a change to `AppSettings` (or a nested type such as
/// `ShortcutSettings`) is not purely additive, i.e. it changes the type or
/// shape of an existing field rather than just adding a new one with
/// `#[serde(default)]`. `load_settings` uses this to decide whether a file
/// written by a different version can be trusted as-is, migrated, or should
/// fall back to defaults.
const CURRENT_SETTINGS_VERSION: u32 = 1;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AppSettings {
    #[serde(default = "current_settings_version")]
    pub version: u32,
    pub allow_cursor_exit_active_displays: bool,
    pub show_overlay_hidden_apps: bool,
    pub shortcut_settings: ShortcutSettings,
    pub shortcut_defaults_initialized: bool,
}

fn current_settings_version() -> u32 {
    CURRENT_SETTINGS_VERSION
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            version: CURRENT_SETTINGS_VERSION,
            allow_cursor_exit_active_displays: true,
            show_overlay_hidden_apps: true,
            shortcut_settings: ShortcutSettings::default(),
            shortcut_defaults_initialized: false,
        }
    }
}

/// Brings a freshly-parsed `AppSettings` up to `CURRENT_SETTINGS_VERSION` in
/// place. Additive changes (new fields with `#[serde(default)]`) never need
/// an entry here since serde already fills them in; add a match arm when a
/// future version changes the shape of an existing field instead. Settings
/// from a newer, not-yet-understood version are left untouched rather than
/// guessed at.
fn migrate_settings(mut settings: AppSettings) -> AppSettings {
    if settings.version < CURRENT_SETTINGS_VERSION {
        settings.version = CURRENT_SETTINGS_VERSION;
    }

    settings
}

pub fn load_settings(app: &AppHandle) -> Result<AppSettings, String> {
    let path = settings_file_path(app)?;

    if !path.exists() {
        return Ok(AppSettings::default());
    }

    let content = fs::read_to_string(&path)
        .map_err(|error| format!("Failed to read settings file {}: {error}", path.display()))?;

    match serde_json::from_str::<AppSettings>(&content) {
        Ok(settings) => Ok(migrate_settings(settings)),
        Err(parse_error) => {
            error!(
                "Failed to parse settings file {}: {parse_error}. Resetting to defaults.",
                path.display()
            );

            let backup_path = path.with_extension("json.bak");
            if let Err(rename_error) = fs::rename(&path, &backup_path) {
                warn!(
                    "Failed to move corrupt settings file {} to {}: {rename_error}",
                    path.display(),
                    backup_path.display()
                );
            }

            Ok(AppSettings::default())
        }
    }
}

pub fn save_settings(app: &AppHandle, settings: &AppSettings) -> Result<(), String> {
    let path = settings_file_path(app)?;
    let parent = path.parent().ok_or_else(|| {
        format!(
            "Missing parent directory for settings file {}",
            path.display()
        )
    })?;

    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "Failed to create settings directory {}: {error}",
            parent.display()
        )
    })?;

    let content = serde_json::to_string_pretty(settings)
        .map_err(|error| format!("Failed to serialize settings: {error}"))?;

    let temp_path = path.with_extension("json.tmp");

    fs::write(&temp_path, content).map_err(|error| {
        format!(
            "Failed to write temporary settings file {}: {error}",
            temp_path.display()
        )
    })?;

    fs::rename(&temp_path, &path).map_err(|error| {
        format!(
            "Failed to replace settings file {} with {}: {error}",
            path.display(),
            temp_path.display()
        )
    })
}

fn settings_file_path(app: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Failed to resolve app data directory: {error}"))?;

    Ok(app_data_dir.join("settings.json"))
}
