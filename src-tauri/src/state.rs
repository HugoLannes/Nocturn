use serde::Serialize;
use std::{collections::HashMap, time::Instant};

use crate::settings::AppSettings;

#[derive(Clone, Debug)]
pub struct DisplayState {
    pub id: String,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub scale_factor: f64,
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
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub scale_factor: f64,
    pub is_primary: bool,
    pub is_blacked_out: bool,
    pub hosts_panel: bool,
    pub can_blackout: bool,
    pub hidden_apps: Vec<HiddenAppSummary>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayUpdatePayload {
    pub displays: Vec<DisplayInfo>,
    pub active_display_count: usize,
    pub blackout_count: usize,
    pub allow_cursor_exit_active_displays: bool,
}

pub struct NocturnState {
    pub displays: HashMap<String, DisplayState>,
    pub settings: AppSettings,
    pub shortcut_registered: bool,
    pub toggle_in_progress: bool,
    pub last_space_press_at: Option<Instant>,
}

impl Default for NocturnState {
    fn default() -> Self {
        Self {
            displays: HashMap::new(),
            settings: AppSettings::default(),
            shortcut_registered: false,
            toggle_in_progress: false,
            last_space_press_at: None,
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

    pub fn blackout_count(&self) -> usize {
        self.displays
            .values()
            .filter(|display| display.is_blacked_out)
            .count()
    }
}
