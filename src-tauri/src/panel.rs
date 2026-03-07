use tauri::{AppHandle, Manager, Monitor, PhysicalPosition, Position};

use crate::state::DisplayState;

pub const PANEL_WIDTH: i32 = 420;
pub const PANEL_HEIGHT: i32 = 640;

pub fn build_display_id(name: Option<&String>, x: i32, y: i32, width: u32, height: u32) -> String {
    let normalized_name = name.cloned().unwrap_or_else(|| "display".to_string());
    format!("{normalized_name}:{x}:{y}:{width}:{height}")
}

pub fn display_id_from_monitor(monitor: &Monitor) -> String {
    let position = monitor.position();
    let size = monitor.size();
    build_display_id(
        monitor.name(),
        position.x,
        position.y,
        size.width,
        size.height,
    )
}

pub fn show_panel(app: &AppHandle) -> Result<(), String> {
    let panel = app
        .get_webview_window("main")
        .ok_or_else(|| "Main panel window is missing.".to_string())?;

    panel.center().map_err(|error| error.to_string())?;
    panel.show().map_err(|error| error.to_string())?;
    panel.set_focus().map_err(|error| error.to_string())?;
    Ok(())
}

pub fn hide_panel(app: &AppHandle) -> Result<(), String> {
    let Some(panel) = app.get_webview_window("main") else {
        return Ok(());
    };

    panel.hide().map_err(|error| error.to_string())
}

pub fn move_panel_to_display(app: &AppHandle, display: &DisplayState) -> Result<(), String> {
    let panel = app
        .get_webview_window("main")
        .ok_or_else(|| "Main panel window is missing.".to_string())?;

    let target_x = display.x + (display.width as i32 - PANEL_WIDTH) / 2;
    let target_y = display.y + (display.height as i32 - PANEL_HEIGHT) / 2;

    panel
        .set_position(Position::Physical(PhysicalPosition::new(
            target_x.max(display.x),
            target_y.max(display.y),
        )))
        .map_err(|error| error.to_string())?;

    Ok(())
}
