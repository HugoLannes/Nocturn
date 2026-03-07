use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use log::info;
use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutEvent, ShortcutState};

use crate::{commands, state::NocturnState};

const DOUBLE_SPACE_WINDOW: Duration = Duration::from_millis(350);

pub fn sync_space_shortcut(
    app: &AppHandle,
    state: &Arc<Mutex<NocturnState>>,
) -> Result<(), String> {
    let (should_register, is_registered) = {
        let state = state.lock().expect("shortcut state poisoned");
        (state.blackout_count() > 0, state.shortcut_registered)
    };

    if should_register && !is_registered {
        let shortcut_state = Arc::clone(state);
        info!("sync_space_shortcut: registering Space shortcut");
        app.global_shortcut()
            .on_shortcut("Space", move |app, _shortcut, event: ShortcutEvent| {
                if event.state == ShortcutState::Pressed {
                    handle_space_press(app, &shortcut_state);
                }
            })
            .map_err(|error| error.to_string())?;

        let mut state = state.lock().expect("shortcut state poisoned");
        state.shortcut_registered = true;
    } else if !should_register && is_registered {
        info!("sync_space_shortcut: unregistering Space shortcut");
        app.global_shortcut()
            .unregister("Space")
            .map_err(|error| error.to_string())?;

        let mut state = state.lock().expect("shortcut state poisoned");
        state.shortcut_registered = false;
        state.last_space_press_at = None;
    }

    Ok(())
}

fn handle_space_press(app: &AppHandle, state: &Arc<Mutex<NocturnState>>) {
    let should_unblank = {
        let mut state = state.lock().expect("shortcut state poisoned");
        let now = Instant::now();

        let should_unblank = state
            .last_space_press_at
            .map(|last_press| now.duration_since(last_press) <= DOUBLE_SPACE_WINDOW)
            .unwrap_or(false);

        state.last_space_press_at = Some(now);
        should_unblank
    };

    if should_unblank {
        info!("handle_space_press: triggering wake all");
        let _ = commands::unblank_all_internal(app, state);
    }
}
