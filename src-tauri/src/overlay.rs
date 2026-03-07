use std::{thread, time::Duration};

use log::{error, info};
use tauri::{
    utils::config::Color, AppHandle, Manager, PhysicalPosition, PhysicalSize, Position, Size,
    WebviewUrl, WebviewWindowBuilder,
};

use crate::state::DisplayState;

const FADE_DURATION_MS: u64 = 180;
pub fn overlay_label(display_id: &str) -> String {
    let sanitized: String = display_id
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    format!("overlay-{sanitized}")
}

pub fn show_overlay(app: &AppHandle, display: &DisplayState) -> Result<(), String> {
    let label = overlay_label(&display.id);
    let width = display.width;
    let height = display.height;
    let x = display.x;
    let y = display.y;

    info!(
        "show_overlay: label={}, physical=({}, {}) {}x{}, scale={}",
        label, display.x, display.y, display.width, display.height, display.scale_factor
    );

    if app.get_webview_window(&label).is_some() {
        if let Some(w) = app.get_webview_window(&label) {
            w.show().map_err(|e| e.to_string())?;
        }
        return Ok(());
    }

    let app_clone = app.clone();
    let label_clone = label.clone();

    thread::spawn(move || {
        let app_for_build = app_clone.clone();
        if let Err(e) = app_clone.run_on_main_thread(move || {
            let result = WebviewWindowBuilder::new(
                &app_for_build,
                &label_clone,
                WebviewUrl::App("overlay.html".into()),
            )
            .decorations(false)
            .always_on_top(true)
            .transparent(true)
            .skip_taskbar(true)
            .focused(false)
            .resizable(false)
            .closable(false)
            .visible(false)
            .shadow(false)
            .inner_size(64.0, 64.0)
            .position(0.0, 0.0)
            .background_color(Color(0, 0, 0, 0))
            .build();

            match result {
                Ok(window) => {
                    if let Err(e) = window.set_ignore_cursor_events(false) {
                        error!(
                            "show_overlay: failed to capture cursor events for {}: {}",
                            label_clone, e
                        );
                    }

                    if let Err(e) =
                        window.set_size(Size::Physical(PhysicalSize::new(width, height)))
                    {
                        error!(
                            "show_overlay: failed to set size for {}: {}",
                            label_clone, e
                        );
                    }

                    if let Err(e) =
                        window.set_position(Position::Physical(PhysicalPosition::new(x, y)))
                    {
                        error!(
                            "show_overlay: failed to set position for {}: {}",
                            label_clone, e
                        );
                    }

                    if let Err(e) = window.show() {
                        error!("show_overlay: failed to show {}: {}", label_clone, e);
                    }

                    info!("show_overlay: window {} created", label_clone);
                }
                Err(e) => error!(
                    "show_overlay: failed to create window {}: {}",
                    label_clone, e
                ),
            }
        }) {
            error!("show_overlay: failed to schedule on main thread: {}", e);
        }
    });

    info!("show_overlay: scheduled {}", label);
    Ok(())
}

pub fn close_overlay(app: &AppHandle, display_id: &str) -> Result<(), String> {
    let label = overlay_label(display_id);
    info!("close_overlay: {}", label);

    if let Some(window) = app.get_webview_window(&label) {
        window
            .eval("window.startFadeOut?.()")
            .map_err(|e| e.to_string())?;

        let app_clone = app.clone();
        let label_clone = label.clone();

        thread::spawn(move || {
            thread::sleep(Duration::from_millis(FADE_DURATION_MS));

            let app_for_destroy = app_clone.clone();
            let destroy_label = label_clone.clone();
            if let Err(e) = app_clone.run_on_main_thread(move || {
                if let Some(window) = app_for_destroy.get_webview_window(&label_clone) {
                    if let Err(e) = window.destroy() {
                        error!("close_overlay: failed to destroy {}: {}", label_clone, e);
                    }
                }
            }) {
                error!(
                    "close_overlay: failed to schedule destroy for {}: {}",
                    destroy_label, e
                );
            }
        });
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
