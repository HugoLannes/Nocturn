use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
    time::Instant,
};

use log::{error, info, warn};
use tauri::{command, AppHandle, Emitter, Manager, Monitor, State};

use crate::{
    cursor, overlay, panel, settings, shortcut,
    state::{DisplayInfo, DisplayState, DisplayUpdatePayload, NocturnState},
};

pub type SharedState = Arc<Mutex<NocturnState>>;

struct ToggleGuard<'a>(&'a SharedState);

impl<'a> ToggleGuard<'a> {
    fn acquire(state: &'a SharedState) -> Result<Self, String> {
        let mut s = state.lock().expect("toggle state poisoned");
        if s.toggle_in_progress {
            return Err("Another display operation is already in progress.".to_string());
        }
        s.toggle_in_progress = true;
        Ok(Self(state))
    }
}

impl Drop for ToggleGuard<'_> {
    fn drop(&mut self) {
        let mut s = self.0.lock().expect("toggle state poisoned");
        s.toggle_in_progress = false;
    }
}

#[command]
pub fn get_displays(
    app: AppHandle,
    state: State<'_, SharedState>,
) -> Result<DisplayUpdatePayload, String> {
    ensure_displays_loaded(&app, state.inner())?;
    sync_runtime_behaviors(&app, state.inner())?;
    build_payload(&app, state.inner())
}

#[command]
pub fn set_allow_cursor_exit_active_displays(
    app: AppHandle,
    state: State<'_, SharedState>,
    allowed: bool,
) -> Result<DisplayUpdatePayload, String> {
    let previous_settings;
    let next_settings;

    {
        let mut state = state.lock().expect("state poisoned");
        previous_settings = state.settings.clone();
        state.settings.allow_cursor_exit_active_displays = allowed;
        next_settings = state.settings.clone();
    }

    if let Err(error) = settings::save_settings(&app, &next_settings) {
        let mut state = state.lock().expect("state poisoned");
        state.settings = previous_settings;
        return Err(error);
    }

    sync_runtime_behaviors(&app, state.inner())?;
    emit_displays_update(&app, state.inner())?;
    build_payload(&app, state.inner())
}

#[command]
pub fn toggle_display(
    app: AppHandle,
    state: State<'_, SharedState>,
    id: String,
) -> Result<String, String> {
    info!("toggle_display: start id={}", id);
    let _guard = ToggleGuard::acquire(state.inner())?;
    let started_at = Instant::now();
    let result = toggle_display_internal(&app, state.inner(), &id);
    match &result {
        Ok(message) => info!(
            "toggle_display: success id={}, message={}, elapsed={}ms",
            id,
            message,
            started_at.elapsed().as_millis()
        ),
        Err(error) => error!(
            "toggle_display: failed id={}, error={}, elapsed={}ms",
            id,
            error,
            started_at.elapsed().as_millis()
        ),
    }
    result
}

#[command]
pub fn unblank_all(app: AppHandle, state: State<'_, SharedState>) -> Result<(), String> {
    let started_at = Instant::now();
    info!("unblank_all: start");
    let result = unblank_all_internal(&app, state.inner());
    match &result {
        Ok(_) => info!(
            "unblank_all: success in {}ms",
            started_at.elapsed().as_millis()
        ),
        Err(error) => error!(
            "unblank_all: failed after {}ms: {}",
            started_at.elapsed().as_millis(),
            error
        ),
    }
    result
}

#[command]
pub fn hide_window(app: AppHandle) -> Result<(), String> {
    panel::hide_panel(&app)
}

#[command]
pub fn close_app(app: AppHandle) {
    app.exit(0);
}

pub fn unblank_all_internal(app: &AppHandle, state: &SharedState) -> Result<(), String> {
    let started_at = Instant::now();

    let blacked_out_ids = {
        let state = state.lock().expect("state poisoned");
        state
            .displays
            .values()
            .filter(|display| display.is_blacked_out)
            .map(|display| display.id.clone())
            .collect::<Vec<_>>()
    };

    overlay::close_all_overlays(app, blacked_out_ids.clone())?;

    {
        let mut state = state.lock().expect("state poisoned");
        for display_id in &blacked_out_ids {
            if let Some(display) = state.displays.get_mut(display_id) {
                display.is_blacked_out = false;
            }
        }
    }

    sync_runtime_behaviors(app, state)?;
    emit_displays_update(app, state)?;

    info!(
        "unblank_all_internal: done in {}ms",
        started_at.elapsed().as_millis()
    );

    Ok(())
}

fn toggle_display_internal(
    app: &AppHandle,
    state: &SharedState,
    id: &str,
) -> Result<String, String> {
    let started_at = Instant::now();
    ensure_displays_loaded(app, state)?;

    let target = {
        let state = state.lock().expect("state poisoned");
        state.displays.get(id).cloned().ok_or_else(|| {
            error!("toggle_display_internal: display not found: {}", id);
            "Display not found.".to_string()
        })?
    };

    info!(
        "toggle_display_internal: id={}, target=({}, {}) {}x{}, blacked_out={}",
        target.id, target.x, target.y, target.width, target.height, target.is_blacked_out
    );

    if target.is_blacked_out {
        info!("toggle_display_internal: restoring display {}", id);
        overlay::close_overlay(app, id)?;

        {
            let mut state = state.lock().expect("state poisoned");
            if let Some(display) = state.displays.get_mut(id) {
                display.is_blacked_out = false;
            }
        }

        sync_runtime_behaviors(app, state)?;
        emit_displays_update(app, state)?;
        info!(
            "toggle_display_internal: restore done id={} in {}ms",
            id,
            started_at.elapsed().as_millis()
        );
        return Ok("Display restored.".to_string());
    }

    let active_display_count = {
        let state = state.lock().expect("state poisoned");
        state.active_display_count()
    };

    info!(
        "toggle_display_internal: active_display_count={}",
        active_display_count
    );

    if active_display_count <= 1 {
        warn!(
            "toggle_display_internal: refusing blackout, only {} active display(s)",
            active_display_count
        );
        return Err("At least one display must stay active.".to_string());
    }

    if panel_is_on_display(app, &target) {
        info!("toggle_display_internal: panel is on target display, moving to fallback");
        let fallback = choose_fallback_display(state, id)?;
        info!(
            "toggle_display_internal: fallback id={}, pos=({}, {}), size={}x{}",
            fallback.id, fallback.x, fallback.y, fallback.width, fallback.height
        );
        panel::move_panel_to_display(app, &fallback)?;
    }

    info!(
        "toggle_display_internal: creating overlay for display {}",
        id
    );
    overlay::show_overlay(app, &target).map_err(|e| {
        error!("toggle_display_internal: overlay creation failed: {}", e);
        e
    })?;

    {
        let mut state = state.lock().expect("state poisoned");
        if let Some(display) = state.displays.get_mut(id) {
            display.is_blacked_out = true;
        }
    }

    sync_runtime_behaviors(app, state)?;
    emit_displays_update(app, state)?;

    info!(
        "toggle_display_internal: display {} blacked out successfully in {}ms",
        id,
        started_at.elapsed().as_millis()
    );
    Ok("Display blacked out.".to_string())
}

fn sync_runtime_behaviors(app: &AppHandle, state: &SharedState) -> Result<(), String> {
    shortcut::sync_space_shortcut(app, state)?;
    cursor::sync_cursor_confinement(state);
    Ok(())
}

fn emit_displays_update(app: &AppHandle, state: &SharedState) -> Result<(), String> {
    let payload = build_payload(app, state)?;
    app.emit("displays-update", payload)
        .map_err(|error| error.to_string())
}

fn build_payload(app: &AppHandle, state: &SharedState) -> Result<DisplayUpdatePayload, String> {
    let state = state.lock().expect("state poisoned");
    let active_display_count = state.active_display_count();
    let blackout_count = state.blackout_count();
    let panel_display_id = current_panel_display_id(app, &state.displays);

    let mut displays = state
        .displays
        .values()
        .cloned()
        .map(|display| DisplayInfo {
            can_blackout: display.is_blacked_out || active_display_count > 1,
            hosts_panel: panel_display_id.as_deref() == Some(display.id.as_str()),
            id: display.id,
            name: display.name,
            width: display.width,
            height: display.height,
            x: display.x,
            y: display.y,
            scale_factor: display.scale_factor,
            is_primary: display.is_primary,
            is_blacked_out: display.is_blacked_out,
        })
        .collect::<Vec<_>>();

    displays.sort_by_key(|display| (display.y, display.x));

    Ok(DisplayUpdatePayload {
        displays,
        active_display_count,
        blackout_count,
        allow_cursor_exit_active_displays: state.settings.allow_cursor_exit_active_displays,
    })
}

fn ensure_displays_loaded(app: &AppHandle, state: &SharedState) -> Result<(), String> {
    let has_displays = {
        let state = state.lock().expect("state poisoned");
        !state.displays.is_empty()
    };

    if has_displays {
        Ok(())
    } else {
        refresh_displays(app, state)
    }
}

fn current_panel_display_id(
    app: &AppHandle,
    displays: &HashMap<String, DisplayState>,
) -> Option<String> {
    let Some(panel) = app.get_webview_window("main") else {
        return None;
    };

    if !panel.is_visible().unwrap_or(false) {
        return None;
    }

    let Ok(pos) = panel.outer_position() else {
        warn!("current_panel_display_id: failed to get panel outer position");
        return None;
    };

    let center_x = pos.x + (panel::PANEL_WIDTH / 2);
    let center_y = pos.y + (panel::PANEL_HEIGHT / 2);

    let result = displays
        .values()
        .find(|display| {
            center_x >= display.x
                && center_x < display.x + display.width as i32
                && center_y >= display.y
                && center_y < display.y + display.height as i32
        })
        .map(|display| display.id.clone());

    result
}

fn choose_fallback_display(state: &SharedState, target_id: &str) -> Result<DisplayState, String> {
    let state = state.lock().expect("state poisoned");
    let target = state
        .displays
        .get(target_id)
        .ok_or_else(|| "Display not found.".to_string())?;

    let fallback = state
        .displays
        .values()
        .filter(|display| !display.is_blacked_out && display.id != target_id)
        .min_by_key(|display| display_distance(target, display))
        .cloned()
        .ok_or_else(|| "At least one display must stay active.".to_string())?;

    Ok(fallback)
}

fn panel_is_on_display(app: &AppHandle, display: &DisplayState) -> bool {
    let Some(panel) = app.get_webview_window("main") else {
        return false;
    };

    if !panel.is_visible().unwrap_or(false as bool) {
        return false;
    }

    let Ok(pos) = panel.outer_position() else {
        warn!("panel_is_on_display: failed to get panel outer position");
        return false;
    };

    // Use the panel center to determine which display it belongs to
    let center_x = pos.x + (panel::PANEL_WIDTH / 2);
    let center_y = pos.y + (panel::PANEL_HEIGHT / 2);

    let is_on_display = center_x >= display.x
        && center_x < display.x + display.width as i32
        && center_y >= display.y
        && center_y < display.y + display.height as i32;

    is_on_display
}

fn display_distance(source: &DisplayState, target: &DisplayState) -> i64 {
    let source_center_x = source.x + (source.width as i32 / 2);
    let source_center_y = source.y + (source.height as i32 / 2);
    let target_center_x = target.x + (target.width as i32 / 2);
    let target_center_y = target.y + (target.height as i32 / 2);

    let dx = source_center_x - target_center_x;
    let dy = source_center_y - target_center_y;

    (dx as i64 * dx as i64) + (dy as i64 * dy as i64)
}

fn refresh_displays(app: &AppHandle, state: &SharedState) -> Result<(), String> {
    let started_at = Instant::now();
    let monitors = app
        .available_monitors()
        .map_err(|error| error.to_string())?;
    let primary_id = app
        .primary_monitor()
        .map_err(|error| error.to_string())?
        .as_ref()
        .map(panel::display_id_from_monitor);

    let next_displays = monitors
        .iter()
        .map(|monitor| monitor_to_display_state(monitor, primary_id.as_deref()))
        .collect::<Vec<_>>();

    let next_ids = next_displays
        .iter()
        .map(|display| display.id.clone())
        .collect::<HashSet<_>>();

    let removed_blackout_ids = {
        let state = state.lock().expect("state poisoned");
        state
            .displays
            .iter()
            .filter_map(|(id, display)| {
                if display.is_blacked_out && !next_ids.contains(id) {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    };

    let previous_blackout_map = {
        let state = state.lock().expect("state poisoned");
        state
            .displays
            .iter()
            .map(|(id, display)| (id.clone(), display.is_blacked_out))
            .collect::<HashMap<_, _>>()
    };

    {
        let mut state = state.lock().expect("state poisoned");
        state.displays = next_displays
            .into_iter()
            .map(|mut display| {
                display.is_blacked_out = previous_blackout_map
                    .get(&display.id)
                    .copied()
                    .unwrap_or(false);
                (display.id.clone(), display)
            })
            .collect::<HashMap<_, _>>();
    }

    if !removed_blackout_ids.is_empty() {
        info!(
            "refresh_displays: closing {} removed overlay(s)",
            removed_blackout_ids.len(),
        );
        overlay::close_all_overlays(app, removed_blackout_ids)?;
    }

    info!(
        "refresh_displays: loaded {} display(s) in {}ms",
        monitors.len(),
        started_at.elapsed().as_millis()
    );

    Ok(())
}

fn monitor_to_display_state(monitor: &Monitor, primary_id: Option<&str>) -> DisplayState {
    let position = monitor.position();
    let size = monitor.size();
    let id = panel::display_id_from_monitor(monitor);

    DisplayState {
        id: id.clone(),
        name: monitor
            .name()
            .cloned()
            .unwrap_or_else(|| format!("Display {}", position.x)),
        width: size.width,
        height: size.height,
        x: position.x,
        y: position.y,
        scale_factor: monitor.scale_factor(),
        is_primary: primary_id == Some(id.as_str()),
        is_blacked_out: false,
    }
}
