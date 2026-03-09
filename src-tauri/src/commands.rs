use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
    time::Instant,
};

use log::{error, info, warn};
use tauri::{command, AppHandle, Emitter, Manager, Monitor, State};

use crate::{
    cursor, overlay, panel, settings, shortcuts,
    state::{
        DisplayInfo, DisplayShortcutBindingInfo, DisplayState, DisplayUpdatePayload, NocturnState,
        ShortcutAction, ShortcutSettingsPayload,
    },
    window_inventory,
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
    let _ = shortcuts::ensure_default_shortcuts(&app, state.inner())?;
    sync_runtime_behaviors(state.inner());
    build_payload(&app, state.inner())
}

#[command]
pub fn get_overlay_card_presentation(window_label: String) -> Option<overlay::OverlayPresentation> {
    overlay::get_overlay_card_presentation(&window_label)
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

    sync_runtime_behaviors(state.inner());
    emit_displays_update(&app, state.inner())?;
    build_payload(&app, state.inner())
}

#[command]
pub fn set_show_overlay_hidden_apps(
    app: AppHandle,
    state: State<'_, SharedState>,
    enabled: bool,
) -> Result<DisplayUpdatePayload, String> {
    let previous_settings;
    let next_settings;

    {
        let mut state = state.lock().expect("state poisoned");
        previous_settings = state.settings.clone();
        state.settings.show_overlay_hidden_apps = enabled;
        next_settings = state.settings.clone();
    }

    if let Err(error) = settings::save_settings(&app, &next_settings) {
        let mut state = state.lock().expect("state poisoned");
        state.settings = previous_settings;
        return Err(error);
    }

    emit_displays_update(&app, state.inner())?;
    build_payload(&app, state.inner())
}

#[command]
pub fn set_shortcut_settings(
    app: AppHandle,
    state: State<'_, SharedState>,
    hotkeys: settings::ShortcutSettings,
) -> Result<DisplayUpdatePayload, String> {
    ensure_displays_loaded(&app, state.inner())?;

    let previous_settings = {
        let state = state.lock().expect("state poisoned");
        state.settings.clone()
    };

    let sanitized_shortcut_settings =
        shortcuts::sanitize_shortcut_settings(state.inner(), hotkeys)?;
    let next_settings = settings::AppSettings {
        shortcut_settings: sanitized_shortcut_settings,
        shortcut_defaults_initialized: true,
        ..previous_settings.clone()
    };

    {
        let mut state = state.lock().expect("state poisoned");
        state.settings = next_settings.clone();
    }

    if let Err(error) = shortcuts::sync_registered_shortcuts(&app, state.inner()) {
        restore_previous_settings(&app, state.inner(), &previous_settings);
        return Err(error);
    }

    if let Err(error) = settings::save_settings(&app, &next_settings) {
        restore_previous_settings(&app, state.inner(), &previous_settings);
        return Err(error);
    }

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
pub fn focus_primary(app: AppHandle, state: State<'_, SharedState>) -> Result<String, String> {
    let started_at = Instant::now();
    info!("focus_primary: start");
    let _guard = ToggleGuard::acquire(state.inner())?;
    let result = focus_primary_internal(&app, state.inner());
    match &result {
        Ok(message) => info!(
            "focus_primary: success message={}, elapsed={}ms",
            message,
            started_at.elapsed().as_millis()
        ),
        Err(error) => error!(
            "focus_primary: failed error={}, elapsed={}ms",
            error,
            started_at.elapsed().as_millis()
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

    sync_runtime_behaviors(state);
    emit_displays_update(app, state)?;

    info!(
        "unblank_all_internal: done in {}ms",
        started_at.elapsed().as_millis()
    );

    Ok(())
}

pub fn focus_primary_internal(app: &AppHandle, state: &SharedState) -> Result<String, String> {
    let started_at = Instant::now();
    ensure_displays_loaded(app, state)?;

    let primary_display = {
        let state = state.lock().expect("state poisoned");
        state
            .displays
            .values()
            .find(|display| display.is_primary)
            .cloned()
            .ok_or_else(|| "Primary display not found.".to_string())?
    };

    let secondary_targets = {
        let state = state.lock().expect("state poisoned");
        state
            .displays
            .values()
            .filter(|display| !display.is_primary && !display.is_blacked_out)
            .cloned()
            .collect::<Vec<_>>()
    };

    if !primary_display.is_blacked_out && secondary_targets.is_empty() {
        info!(
            "focus_primary_internal: already focused in {}ms",
            started_at.elapsed().as_millis()
        );
        return Ok("Focus mode already active.".to_string());
    }

    if !panel_is_on_display(app, &primary_display) {
        panel::move_panel_to_display(app, &primary_display)?;
    }

    if primary_display.is_blacked_out {
        info!(
            "focus_primary_internal: restoring primary display {}",
            primary_display.id
        );
        overlay::close_overlay(app, &primary_display.id)?;

        let mut state = state.lock().expect("state poisoned");
        if let Some(display) = state.displays.get_mut(&primary_display.id) {
            display.is_blacked_out = false;
        }
    }

    for display in &secondary_targets {
        info!(
            "focus_primary_internal: blacking out secondary display {}",
            display.id
        );
        overlay::show_overlay(app, display)?;
    }

    if !secondary_targets.is_empty() {
        let secondary_ids = secondary_targets
            .iter()
            .map(|display| display.id.clone())
            .collect::<HashSet<_>>();

        let mut state = state.lock().expect("state poisoned");
        for display_id in &secondary_ids {
            if let Some(display) = state.displays.get_mut(display_id) {
                display.is_blacked_out = true;
            }
        }
    }

    sync_runtime_behaviors(state);
    emit_displays_update(app, state)?;

    info!(
        "focus_primary_internal: done restored_primary={} secondary_count={} elapsed={}ms",
        primary_display.is_blacked_out,
        secondary_targets.len(),
        started_at.elapsed().as_millis()
    );

    Ok("Focus mode enabled.".to_string())
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

        sync_runtime_behaviors(state);
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

    sync_runtime_behaviors(state);
    emit_displays_update(app, state)?;

    info!(
        "toggle_display_internal: display {} blacked out successfully in {}ms",
        id,
        started_at.elapsed().as_millis()
    );
    Ok("Display blacked out.".to_string())
}

pub(crate) fn execute_shortcut_action(
    app: &AppHandle,
    state: &SharedState,
    action: ShortcutAction,
) -> Result<(), String> {
    let _guard = ToggleGuard::acquire(state)?;

    match action {
        ShortcutAction::FocusMode => focus_primary_internal(app, state).map(|_| ()),
        ShortcutAction::ToggleDisplay { display_key } => {
            ensure_displays_loaded(app, state)?;

            let display_id = {
                let state = state.lock().expect("state poisoned");
                state
                    .displays
                    .values()
                    .find(|display| display.persistent_key == display_key)
                    .map(|display| display.id.clone())
                    .ok_or_else(|| {
                        "Display for this shortcut is not currently available.".to_string()
                    })?
            };

            toggle_display_internal(app, state, &display_id).map(|_| ())
        }
    }
}

fn sync_runtime_behaviors(state: &SharedState) {
    cursor::sync_cursor_confinement(state);
}

pub fn refresh_display_snapshot(app: &AppHandle, state: &SharedState) -> Result<(), String> {
    refresh_displays(app, state)?;
    let shortcuts_initialized = shortcuts::ensure_default_shortcuts(app, state)?;
    if !shortcuts_initialized {
        shortcuts::sync_registered_shortcuts(app, state)?;
    }
    sync_runtime_behaviors(state);
    emit_displays_update(app, state)
}

fn restore_previous_settings(
    app: &AppHandle,
    state: &SharedState,
    previous_settings: &settings::AppSettings,
) {
    {
        let mut state = state.lock().expect("state poisoned");
        state.settings = previous_settings.clone();
    }

    if let Err(error) = shortcuts::sync_registered_shortcuts(app, state) {
        error!(
            "restore_previous_settings: failed to restore shortcuts: {}",
            error
        );
    }
}

fn emit_displays_update(app: &AppHandle, state: &SharedState) -> Result<(), String> {
    let payload = build_payload(app, state)?;
    app.emit("displays-update", payload)
        .map_err(|error| error.to_string())
}

fn build_payload(app: &AppHandle, state: &SharedState) -> Result<DisplayUpdatePayload, String> {
    let (
        displays_map,
        allow_cursor_exit_active_displays,
        show_overlay_hidden_apps,
        shortcut_settings,
    ) = {
        let state = state.lock().expect("state poisoned");
        (
            state.displays.clone(),
            state.settings.allow_cursor_exit_active_displays,
            state.settings.show_overlay_hidden_apps,
            state.settings.shortcut_settings.clone(),
        )
    };
    let active_display_count = displays_map
        .values()
        .filter(|display| !display.is_blacked_out)
        .count();
    let blackout_count = displays_map
        .values()
        .filter(|display| display.is_blacked_out)
        .count();
    let panel_display_id = current_panel_display_id(app, &displays_map);
    let hidden_apps_by_display = window_inventory::snapshot_hidden_apps_by_display(&displays_map)?;
    let overlay_presentations = build_overlay_presentations(
        &displays_map,
        &hidden_apps_by_display,
        show_overlay_hidden_apps,
    );
    overlay::sync_overlay_cards(app, &displays_map, overlay_presentations)?;
    let available_display_keys = displays_map
        .values()
        .map(|display| display.persistent_key.as_str())
        .collect::<HashSet<_>>();
    let hotkeys_by_display_key = shortcut_settings
        .display_bindings
        .iter()
        .map(|binding| (binding.display_key.as_str(), binding.accelerator.clone()))
        .collect::<HashMap<_, _>>();
    let mut display_bindings = shortcut_settings
        .display_bindings
        .iter()
        .map(|binding| DisplayShortcutBindingInfo {
            display_key: binding.display_key.clone(),
            display_label: binding.display_label.clone(),
            accelerator: binding.accelerator.clone(),
            is_available: available_display_keys.contains(binding.display_key.as_str()),
        })
        .collect::<Vec<_>>();

    display_bindings.sort_by(|left, right| {
        left.display_label
            .cmp(&right.display_label)
            .then_with(|| left.display_key.cmp(&right.display_key))
    });

    let mut displays = displays_map
        .values()
        .cloned()
        .map(|display| {
            let display_id = display.id.clone();
            DisplayInfo {
                can_blackout: display.is_blacked_out || active_display_count > 1,
                hosts_panel: panel_display_id.as_deref() == Some(display_id.as_str()),
                id: display_id.clone(),
                name: display.name,
                manufacturer: display.manufacturer,
                model: display.model,
                persistent_key: display.persistent_key.clone(),
                width: display.width,
                height: display.height,
                x: display.x,
                y: display.y,
                scale_factor: display.scale_factor,
                refresh_rate: display.refresh_rate,
                orientation: display.orientation,
                is_primary: display.is_primary,
                is_blacked_out: display.is_blacked_out,
                hotkey: hotkeys_by_display_key
                    .get(display.persistent_key.as_str())
                    .cloned(),
                hidden_apps: hidden_apps_by_display
                    .get(&display_id)
                    .cloned()
                    .unwrap_or_default(),
            }
        })
        .collect::<Vec<_>>();

    displays.sort_by_key(|display| (display.y, display.x));

    Ok(DisplayUpdatePayload {
        displays,
        active_display_count,
        blackout_count,
        allow_cursor_exit_active_displays,
        show_overlay_hidden_apps,
        shortcut_settings: ShortcutSettingsPayload {
            focus_mode_hotkey: shortcut_settings.focus_mode_hotkey,
            display_bindings,
        },
    })
}

fn build_overlay_presentations(
    displays: &HashMap<String, DisplayState>,
    hidden_apps_by_display: &HashMap<String, Vec<crate::state::HiddenAppSummary>>,
    show_overlay_hidden_apps: bool,
) -> HashMap<String, overlay::OverlayPresentation> {
    let primary_display = displays
        .values()
        .find(|display| display.is_primary)
        .cloned();

    displays
        .values()
        .filter(|display| display.is_blacked_out)
        .map(|display| {
            let reference_display = primary_display
                .as_ref()
                .filter(|primary| primary.id != display.id)
                .cloned()
                .or_else(|| nearest_active_display(displays, &display.id));

            let dock = reference_display
                .as_ref()
                .map(|reference| overlay_dock_towards(display, reference))
                .unwrap_or(overlay::OverlayDock::Center);

            (
                display.id.clone(),
                overlay::OverlayPresentation {
                    hidden_apps: hidden_apps_by_display
                        .get(&display.id)
                        .cloned()
                        .unwrap_or_default(),
                    dock,
                    is_enabled: show_overlay_hidden_apps,
                },
            )
        })
        .collect()
}

fn nearest_active_display(
    displays: &HashMap<String, DisplayState>,
    source_display_id: &str,
) -> Option<DisplayState> {
    let source = displays.get(source_display_id)?;

    displays
        .values()
        .filter(|display| !display.is_blacked_out && display.id != source_display_id)
        .min_by_key(|display| display_distance(source, display))
        .cloned()
}

fn overlay_dock_towards(source: &DisplayState, target: &DisplayState) -> overlay::OverlayDock {
    let source_center_x = source.x + (source.width as i32 / 2);
    let source_center_y = source.y + (source.height as i32 / 2);
    let target_center_x = target.x + (target.width as i32 / 2);
    let target_center_y = target.y + (target.height as i32 / 2);

    let dx = target_center_x - source_center_x;
    let dy = target_center_y - source_center_y;

    if dx == 0 && dy == 0 {
        return overlay::OverlayDock::Center;
    }

    if dx.abs() > dy.abs() {
        if dx > 0 {
            overlay::OverlayDock::Right
        } else {
            overlay::OverlayDock::Left
        }
    } else if dy > 0 {
        overlay::OverlayDock::Bottom
    } else {
        overlay::OverlayDock::Top
    }
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

    let identities = crate::monitor_info::query_monitor_identities();

    let next_displays = monitors
        .iter()
        .map(|monitor| monitor_to_display_state(monitor, primary_id.as_deref(), &identities))
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

fn monitor_to_display_state(
    monitor: &Monitor,
    primary_id: Option<&str>,
    identities: &HashMap<String, crate::monitor_info::MonitorIdentity>,
) -> DisplayState {
    let position = monitor.position();
    let size = monitor.size();
    let id = panel::display_id_from_monitor(monitor);
    let name = monitor
        .name()
        .cloned()
        .unwrap_or_else(|| format!("Display {}", position.x));

    let identity = identities.get(&name).cloned().unwrap_or_default();
    let (refresh_rate, orientation) = crate::monitor_info::query_display_settings(&name);

    DisplayState {
        id: id.clone(),
        name,
        manufacturer: identity.manufacturer,
        model: identity.model,
        persistent_key: if identity.persistent_key.is_empty() {
            format!("gdi:{}", id)
        } else {
            identity.persistent_key
        },
        width: size.width,
        height: size.height,
        x: position.x,
        y: position.y,
        scale_factor: monitor.scale_factor(),
        refresh_rate,
        orientation,
        is_primary: primary_id == Some(id.as_str()),
        is_blacked_out: false,
    }
}
