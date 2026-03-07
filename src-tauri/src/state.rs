use serde::Serialize;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Instant,
};

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
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayUpdatePayload {
    pub displays: Vec<DisplayInfo>,
    pub active_display_count: usize,
    pub blackout_count: usize,
}

pub struct NocturnState {
    pub displays: HashMap<String, DisplayState>,
    pub shortcut_registered: bool,
    pub toggle_in_progress: bool,
    pub last_space_press_at: Option<Instant>,
    pub cursor_stop_flag: Option<Arc<AtomicBool>>,
}

impl Default for NocturnState {
    fn default() -> Self {
        Self {
            displays: HashMap::new(),
            shortcut_registered: false,
            toggle_in_progress: false,
            last_space_press_at: None,
            cursor_stop_flag: None,
        }
    }
}

impl NocturnState {
    pub fn active_display_count(&self) -> usize {
        self.displays.values().filter(|display| !display.is_blacked_out).count()
    }

    pub fn blackout_count(&self) -> usize {
        self.displays.values().filter(|display| display.is_blacked_out).count()
    }

    pub fn reset_cursor_loop(&mut self) {
        if let Some(stop_flag) = self.cursor_stop_flag.take() {
            stop_flag.store(true, Ordering::Relaxed);
        }
    }
}
