use log::{debug, error, info};
use tauri::{window::WindowBuilder, utils::config::Color, AppHandle, Manager};

use crate::state::DisplayState;

pub fn overlay_label(display_id: &str) -> String {
    let sanitized: String = display_id
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();
    format!("overlay-{sanitized}")
}

pub fn show_overlay(app: &AppHandle, display: &DisplayState) -> Result<(), String> {
    let label = overlay_label(&display.id);
    let scale = display.scale_factor;
    let logical_w = display.width as f64 / scale;
    let logical_h = display.height as f64 / scale;
    let logical_x = display.x as f64 / scale;
    let logical_y = display.y as f64 / scale;

    info!(
        "show_overlay: label={}, physical=({}, {}) {}x{}, logical=({:.0}, {:.0}) {:.0}x{:.0}, scale={}",
        label, display.x, display.y, display.width, display.height,
        logical_x, logical_y, logical_w, logical_h, scale
    );

    if app.get_webview_window(&label).is_some() || app.get_window(&label).is_some() {
        debug!("show_overlay: window {} already exists, showing", label);
        if let Some(w) = app.get_window(&label) {
            w.show().map_err(|e| e.to_string())?;
        }
        return Ok(());
    }

    let app_clone = app.clone();
    let label_clone = label.clone();

    app.run_on_main_thread(move || {
        let result = WindowBuilder::new(&app_clone, &label_clone)
            .decorations(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .focused(false)
            .resizable(false)
            .closable(false)
            .visible(true)
            .inner_size(logical_w, logical_h)
            .position(logical_x, logical_y)
            .background_color(Color(0, 0, 0, 255))
            .build();

        match result {
            Ok(_) => {
                info!("show_overlay: window {} created", label_clone);
                if let Some(panel) = app_clone.get_webview_window("main") {
                    let _ = panel.set_focus();
                }
            }
            Err(e) => error!("show_overlay: failed to create window {}: {}", label_clone, e),
        }
    })
    .map_err(|e| {
        error!("show_overlay: failed to schedule on main thread: {}", e);
        e.to_string()
    })?;

    info!("show_overlay: creation scheduled for {}", label);
    Ok(())
}

pub fn close_overlay(app: &AppHandle, display_id: &str) -> Result<(), String> {
    let label = overlay_label(display_id);
    info!("close_overlay: label={}", label);

    if let Some(window) = app.get_window(&label) {
        window.destroy().map_err(|e| e.to_string())?;
    } else {
        debug!("close_overlay: window {} not found, nothing to close", label);
    }

    Ok(())
}

pub fn close_all_overlays(
    app: &AppHandle,
    display_ids: impl IntoIterator<Item = String>,
) -> Result<(), String> {
    for display_id in display_ids {
        close_overlay(app, &display_id)?;
    }

    Ok(())
}
