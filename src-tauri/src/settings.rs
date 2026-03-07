use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AppSettings {
    pub allow_cursor_exit_active_displays: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            allow_cursor_exit_active_displays: true,
        }
    }
}

pub fn load_settings(app: &AppHandle) -> Result<AppSettings, String> {
    let path = settings_file_path(app)?;

    if !path.exists() {
        return Ok(AppSettings::default());
    }

    let content = fs::read_to_string(&path)
        .map_err(|error| format!("Failed to read settings file {}: {error}", path.display()))?;

    serde_json::from_str(&content)
        .map_err(|error| format!("Failed to parse settings file {}: {error}", path.display()))
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

    fs::write(&path, content)
        .map_err(|error| format!("Failed to write settings file {}: {error}", path.display()))
}

fn settings_file_path(app: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Failed to resolve app data directory: {error}"))?;

    Ok(app_data_dir.join("settings.json"))
}
