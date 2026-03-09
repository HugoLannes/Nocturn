#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod cursor;
mod monitor_info;
mod overlay;
mod panel;
mod settings;
mod shortcuts;
mod state;
mod window_inventory;

use std::sync::{Arc, Mutex};

use commands::SharedState;
use state::NocturnState;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};

fn main() {
    let shared_state: SharedState = Arc::new(Mutex::new(NocturnState::default()));

    tauri::Builder::default()
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
                        let _ = panel::show_panel(tray.app_handle());
                        let state = tray.app_handle().state::<SharedState>();
                        let _ =
                            commands::refresh_display_snapshot(tray.app_handle(), state.inner());
                    }
                })
                .build(app)?;

            app.on_menu_event(|app, event| match event.id().as_ref() {
                "show-panel" => {
                    let _ = panel::show_panel(app);
                    let state = app.state::<SharedState>();
                    let _ = commands::refresh_display_snapshot(app, state.inner());
                }
                "wake-all" => {
                    let state = app.state::<SharedState>();
                    let _ = commands::unblank_all_internal(app, state.inner());
                }
                "quit" => app.exit(0),
                _ => {}
            });

            let panel_window = app
                .get_webview_window("main")
                .ok_or("Missing main panel window")?;

            let app_handle = app.handle().clone();
            panel_window.on_window_event(move |event| {
                if let WindowEvent::CloseRequested { .. } = event {
                    app_handle.exit(0);
                }
            });

            let _ = commands::refresh_display_snapshot(app.handle(), state.inner());

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Nocturn");
}
