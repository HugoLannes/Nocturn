use std::{
    collections::HashMap,
    ffi::OsStr,
    os::windows::ffi::OsStrExt,
    ptr,
    sync::{Mutex, OnceLock},
    thread,
};

use log::{error, info};
use tauri::AppHandle;
use windows_sys::Win32::{
    Foundation::{GetLastError, HWND, LPARAM, LRESULT, WPARAM},
    Graphics::Gdi::{BLACK_BRUSH, GetStockObject, HBRUSH},
    System::LibraryLoader::GetModuleHandleW,
    UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, HTCLIENT, HWND_TOPMOST, IsWindow,
        RegisterClassW, SW_SHOWNA, SWP_NOACTIVATE, SWP_SHOWWINDOW, SetWindowPos, ShowWindow,
        WM_NCHITTEST, WNDCLASSW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
    },
};

use crate::state::DisplayState;

const OVERLAY_CLASS_NAME: &str = "NocturnNativeOverlay";

static OVERLAY_WINDOWS: OnceLock<Mutex<HashMap<String, isize>>> = OnceLock::new();
static OVERLAY_CLASS_ATOM: OnceLock<u16> = OnceLock::new();
static OVERLAY_CLASS_NAME_WIDE: OnceLock<Vec<u16>> = OnceLock::new();

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
    let display = display.clone();
    let label = overlay_label(&display.id);

    info!(
        "show_overlay: label={}, physical=({}, {}) {}x{}, scale={}",
        label, display.x, display.y, display.width, display.height, display.scale_factor
    );

    let app = app.clone();
    thread::spawn(move || {
        let display_id = display.id.clone();
        let display_for_create = display.clone();
        let label_for_log = label.clone();

        if let Err(error) = app.run_on_main_thread(move || {
            if let Err(error) = create_or_update_overlay(&display_for_create) {
                error!(
                    "show_overlay: failed to create native overlay {}: {}",
                    label_for_log, error
                );
            }
        }) {
            error!(
                "show_overlay: failed to schedule native overlay {}: {}",
                label, error
            );
            return;
        }

        info!("show_overlay: scheduled native overlay for {}", display_id);
    });

    Ok(())
}

pub fn close_overlay(app: &AppHandle, display_id: &str) -> Result<(), String> {
    let label = overlay_label(display_id);
    info!("close_overlay: {}", label);

    let display_id = display_id.to_string();
    let app = app.clone();

    thread::spawn(move || {
        let label_for_log = label.clone();
        if let Err(error) = app.run_on_main_thread(move || {
            if let Err(error) = destroy_overlay(&display_id) {
                error!(
                    "close_overlay: failed to destroy native overlay {}: {}",
                    label_for_log, error
                );
            }
        }) {
            error!(
                "close_overlay: failed to schedule destroy for {}: {}",
                label, error
            );
        }
    });

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

fn overlay_windows() -> &'static Mutex<HashMap<String, isize>> {
    OVERLAY_WINDOWS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn overlay_class_name_wide() -> &'static [u16] {
    OVERLAY_CLASS_NAME_WIDE
        .get_or_init(|| wide_null(OVERLAY_CLASS_NAME))
        .as_slice()
}

fn create_or_update_overlay(display: &DisplayState) -> Result<(), String> {
    ensure_overlay_class_registered()?;

    let hwnd = {
        let windows = overlay_windows().lock().expect("overlay registry poisoned");
        windows.get(&display.id).copied()
    };

    if let Some(hwnd) = hwnd {
        let hwnd = hwnd as HWND;
        if unsafe { IsWindow(hwnd) } != 0 {
            unsafe {
                position_overlay(hwnd, display);
            }
            info!(
                "create_or_update_overlay: reused native overlay for {}",
                display.id
            );
            return Ok(());
        }

        overlay_windows()
            .lock()
            .expect("overlay registry poisoned")
            .remove(&display.id);
    }

    let hwnd = unsafe { create_overlay_window(display)? };
    overlay_windows()
        .lock()
        .expect("overlay registry poisoned")
        .insert(display.id.clone(), hwnd as isize);

    info!(
        "create_or_update_overlay: created native overlay for {}",
        display.id
    );
    Ok(())
}

fn destroy_overlay(display_id: &str) -> Result<(), String> {
    let hwnd = overlay_windows()
        .lock()
        .expect("overlay registry poisoned")
        .remove(display_id);

    let Some(hwnd) = hwnd else {
        return Ok(());
    };

    let hwnd = hwnd as HWND;
    if unsafe { IsWindow(hwnd) } == 0 {
        return Ok(());
    }

    let destroy_result = unsafe { DestroyWindow(hwnd) };
    if destroy_result == 0 {
        return Err(last_error("DestroyWindow failed"));
    }

    Ok(())
}

fn ensure_overlay_class_registered() -> Result<(), String> {
    if OVERLAY_CLASS_ATOM.get().is_some() {
        return Ok(());
    }

    let hinstance = unsafe { GetModuleHandleW(ptr::null()) };
    if hinstance.is_null() {
        return Err(last_error("GetModuleHandleW failed"));
    }

    let class_name = overlay_class_name_wide();
    let class = WNDCLASSW {
        style: 0,
        lpfnWndProc: Some(overlay_wnd_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: hinstance,
        hIcon: ptr::null_mut(),
        hCursor: ptr::null_mut(),
        hbrBackground: unsafe { GetStockObject(BLACK_BRUSH) as HBRUSH },
        lpszMenuName: ptr::null(),
        lpszClassName: class_name.as_ptr(),
    };

    let atom = unsafe { RegisterClassW(&class) };
    if atom == 0 {
        return Err(last_error("RegisterClassW failed"));
    }

    let _ = OVERLAY_CLASS_ATOM.set(atom);
    Ok(())
}

unsafe fn create_overlay_window(display: &DisplayState) -> Result<HWND, String> {
    let hinstance = unsafe { GetModuleHandleW(ptr::null()) };
    if hinstance.is_null() {
        return Err(last_error("GetModuleHandleW failed"));
    }

    let title = wide_null(&format!("Nocturn Overlay {}", display.id));
    let hwnd = unsafe {
        CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            overlay_class_name_wide().as_ptr(),
            title.as_ptr(),
            WS_POPUP,
            display.x,
            display.y,
            display.width as i32,
            display.height as i32,
            ptr::null_mut(),
            ptr::null_mut(),
            hinstance,
            ptr::null(),
        )
    };

    if hwnd.is_null() {
        return Err(last_error("CreateWindowExW failed"));
    }

    unsafe {
        position_overlay(hwnd, display);
        ShowWindow(hwnd, SW_SHOWNA);
    }
    Ok(hwnd)
}

unsafe fn position_overlay(hwnd: HWND, display: &DisplayState) {
    unsafe {
        SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            display.x,
            display.y,
            display.width as i32,
            display.height as i32,
            SWP_SHOWWINDOW | SWP_NOACTIVATE,
        );
    }
}

unsafe extern "system" fn overlay_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_NCHITTEST => HTCLIENT as LRESULT,
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

fn wide_null(value: &str) -> Vec<u16> {
    OsStr::new(value)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

fn last_error(context: &str) -> String {
    format!("{} (win32 error {})", context, unsafe { GetLastError() })
}
