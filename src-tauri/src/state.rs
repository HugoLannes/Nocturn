use serde::Serialize;
use std::collections::HashMap;

use crate::settings::AppSettings;

#[derive(Clone, Debug)]
pub struct DisplayState {
    pub id: String,
    pub name: String,
    pub manufacturer: String,
    pub model: String,
    pub persistent_key: String,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub scale_factor: f64,
    pub refresh_rate: u32,
    pub orientation: u32,
    pub is_primary: bool,
    pub is_blacked_out: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HiddenAppSummary {
    pub app_name: String,
    pub window_count: usize,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayInfo {
    pub id: String,
    pub name: String,
    pub manufacturer: String,
    pub model: String,
    pub persistent_key: String,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub scale_factor: f64,
    pub refresh_rate: u32,
    pub orientation: u32,
    pub is_primary: bool,
    pub is_blacked_out: bool,
    pub hosts_panel: bool,
    pub can_blackout: bool,
    pub hotkey: Option<String>,
    pub hidden_apps: Vec<HiddenAppSummary>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayShortcutBindingInfo {
    pub display_key: String,
    pub display_label: String,
    pub accelerator: String,
    pub is_available: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutSettingsPayload {
    pub focus_mode_hotkey: Option<String>,
    pub display_bindings: Vec<DisplayShortcutBindingInfo>,
}

#[derive(Clone, Debug)]
pub enum ShortcutAction {
    FocusMode,
    ToggleDisplay { display_key: String },
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayUpdatePayload {
    pub displays: Vec<DisplayInfo>,
    pub active_display_count: usize,
    pub blackout_count: usize,
    pub allow_cursor_exit_active_displays: bool,
    pub show_overlay_hidden_apps: bool,
    pub shortcut_settings: ShortcutSettingsPayload,
}

pub struct NocturnState {
    pub displays: HashMap<String, DisplayState>,
    pub settings: AppSettings,
    pub toggle_in_progress: bool,
    pub registered_shortcuts: HashMap<u32, ShortcutAction>,
}

impl Default for NocturnState {
    fn default() -> Self {
        Self {
            displays: HashMap::new(),
            settings: AppSettings::default(),
            toggle_in_progress: false,
            registered_shortcuts: HashMap::new(),
        }
    }
}

impl NocturnState {
    pub fn active_display_count(&self) -> usize {
        self.displays
            .values()
            .filter(|display| !display.is_blacked_out)
            .count()
    }
}
