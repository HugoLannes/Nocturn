#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod cursor;
mod display_watcher;
mod monitor_info;
mod overlay;
mod panel;
mod settings;
mod shortcuts;
mod state;
mod window_inventory;

use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use log::error;

use commands::SharedState;
use state::NocturnState;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, WindowEvent,
};

// How often the cursor clip is re-asserted in the background. Windows silently
// drops ClipCursor on secure-desktop switches (UAC prompts, lock screen) and
// whenever another process calls ClipCursor itself, with no notification to
// us, so a lightweight periodic re-apply is the simplest way to keep "cursor
// cannot exit active displays" honest without a WinEventHook.
const CURSOR_CONFINEMENT_REASSERT_INTERVAL: Duration = Duration::from_secs(2);

// Brings the existing panel to the foreground. Shared by the tray icon's left
// click, the "Show Panel" menu item, and the single-instance plugin callback
// (fired when the user relaunches Nocturn while it is already running, e.g.
// from the Start menu, instead of that relaunch spawning a second process).
fn focus_existing_panel(app: &AppHandle) {
    let state = app.state::<SharedState>();
    let _ = commands::relocate_panel_off_blacked_out_display(app, state.inner());
    let _ = panel::show_panel(app);
    let _ = commands::refresh_display_snapshot(app, state.inner());
}

fn main() {
    // Capture the thread `main()` runs on: `.run()` below turns it into the
    // event loop thread, and overlay operations need to recognize it to avoid
    // deadlocking when they're already called from a main-thread event handler
    // (tray/menu clicks, window events).
    overlay::mark_main_thread();

    let shared_state: SharedState = Arc::new(Mutex::new(NocturnState::default()));

    let cursor_confinement_state = shared_state.clone();
    thread::spawn(move || loop {
        thread::sleep(CURSOR_CONFINEMENT_REASSERT_INTERVAL);
        cursor::sync_cursor_confinement(&cursor_confinement_state);
    });

    tauri::Builder::default()
        // Registered first: if another instance of Nocturn is already
        // running, this intercepts the relaunch (Start menu, autostart,
        // double-clicking the installer shortcut, etc.), forwards it here
        // instead of letting a second process start, and this instance's
        // callback simply focuses the existing panel. Without this, a
        // relaunch would spawn a duplicate tray icon and a second process
        // whose global-shortcut registration silently fails against the
        // first (Windows RegisterHotKey is per-process).
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            focus_existing_panel(app);
        }))
        // Without a logger implementation the `log` facade's calls throughout
        // commands.rs/overlay.rs/monitor_info.rs/shortcuts.rs are silently
        // discarded, which left overlay/shortcut failures completely
        // undiagnosable in the field. This writes to stdout (debug builds)
        // and to a rotating file in the app's log dir (all builds).
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .max_file_size(5_000_000)
                .build(),
        )
        .manage(shared_state)
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    let state = app.state::<SharedState>();
                    shortcuts::handle_shortcut_event(app, shortcut, event, state.inner());
                })
                .build(),
        )
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            commands::get_displays,
            commands::get_overlay_card_presentation,
            commands::set_allow_cursor_exit_active_displays,
            commands::set_show_overlay_hidden_apps,
            commands::set_shortcut_settings,
            commands::reset_to_defaults,
            commands::toggle_display,
            commands::unblank_all,
            commands::focus_primary,
            commands::hide_window,
            commands::close_app
        ])
        .setup(|app| {
            let loaded_settings = settings::load_settings(app.handle())?;
            let state = app.state::<SharedState>();
            let tray_icon = app
                .default_window_icon()
                .cloned()
                .ok_or("Missing default app icon")?;

            {
                let mut state = state.lock().expect("state poisoned");
                state.settings = loaded_settings;
            }

            let show_panel = MenuItemBuilder::new("Show Panel")
                .id("show-panel")
                .build(app)?;
            let wake_all = MenuItemBuilder::new("Wake All").id("wake-all").build(app)?;
            let quit = MenuItemBuilder::new("Quit").id("quit").build(app)?;

            let tray_menu = MenuBuilder::new(app)
                .items(&[&show_panel, &wake_all, &quit])
                .build()?;

            TrayIconBuilder::new()
                .icon(tray_icon)
                .tooltip("Nocturn")
                .menu(&tray_menu)
                .show_menu_on_left_click(false)
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        focus_existing_panel(tray.app_handle());
                    }
                })
                .build(app)?;

            app.on_menu_event(|app, event| match event.id().as_ref() {
                "show-panel" => {
                    focus_existing_panel(app);
                }
                "wake-all" => {
                    let state = app.state::<SharedState>();
                    let _ = commands::unblank_all_internal(app, state.inner());
                }
                "quit" => {
                    cursor::release_cursor();
                    app.exit(0);
                }
                _ => {}
            });

            let panel_window = app
                .get_webview_window("main")
                .ok_or("Missing main panel window")?;

            let app_handle = app.handle().clone();
            panel_window.on_window_event(move |event| {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    // Nocturn is tray-resident: an OS-level close request
                    // (Alt+F4 on the frameless window) hides the panel instead
                    // of exiting, mirroring what the custom titlebar's own
                    // Close button does (see commands::close_app), so active
                    // blackout overlays and hotkey registrations survive.
                    // Use "Quit" from the tray context menu to actually exit.
                    api.prevent_close();
                    let _ = panel::hide_panel(&app_handle);
                }
            });

            // Listen for WM_DISPLAYCHANGE/WM_DEVICECHANGE/WM_POWERBROADCAST so the
            // display snapshot is refreshed whenever monitors are hot-plugged,
            // resolution/DPI changes, or the machine resumes from sleep — otherwise
            // blackout and focus-mode logic can operate on stale/phantom monitors.
            if let Err(error) = display_watcher::install(app.handle(), state.inner()) {
                error!("failed to install display watcher: {}", error);
            }

            if let Err(error) = commands::refresh_display_snapshot(app.handle(), state.inner()) {
                error!("failed to refresh display snapshot on startup: {}", error);
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Nocturn");
}
