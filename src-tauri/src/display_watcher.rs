use std::{
    ptr,
    sync::OnceLock,
};

use log::{error, info};
use tauri::AppHandle;
use windows_sys::Win32::{
    Foundation::{GetLastError, HWND, LPARAM, LRESULT, WPARAM},
    System::LibraryLoader::GetModuleHandleW,
    UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, RegisterClassW, DBT_DEVNODES_CHANGED,
        PBT_APMRESUMEAUTOMATIC, PBT_APMRESUMESUSPEND, WM_DEVICECHANGE, WM_DISPLAYCHANGE,
        WM_POWERBROADCAST, WNDCLASSW, WS_EX_TOOLWINDOW, WS_OVERLAPPED,
    },
};

use crate::commands::{self, SharedState};

const WATCHER_CLASS_NAME: &str = "NocturnDisplayWatcher";

static WATCHER_APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();
static WATCHER_STATE: OnceLock<SharedState> = OnceLock::new();
static WATCHER_CLASS_NAME_WIDE: OnceLock<Vec<u16>> = OnceLock::new();
static WATCHER_CLASS_ATOM: OnceLock<u16> = OnceLock::new();

/// Creates a hidden top-level window whose sole purpose is to observe
/// Windows display-topology notifications (monitor hot-plug/unplug,
/// resolution/DPI changes, and resume-from-sleep) and refresh the
/// backend's display snapshot whenever one arrives. Without this, the
/// snapshot only updates on tray/panel interaction, so blackout and
/// focus-mode logic could operate on phantom monitors after a docking
/// change (see issue #12).
///
/// The window is intentionally never shown: `WM_DISPLAYCHANGE`,
/// `WM_DEVICECHANGE`, and `WM_POWERBROADCAST` are broadcast to every
/// top-level window regardless of visibility, but NOT to message-only
/// (`HWND_MESSAGE`) windows, so a real (hidden) top-level window is used.
pub fn install(app: &AppHandle, state: &SharedState) -> Result<(), String> {
    let _ = WATCHER_APP_HANDLE.set(app.clone());
    let _ = WATCHER_STATE.set(state.clone());

    ensure_watcher_class_registered()?;

    let hinstance = unsafe { GetModuleHandleW(ptr::null()) };
    if hinstance.is_null() {
        return Err(last_error("GetModuleHandleW failed"));
    }

    let title = wide_null(WATCHER_CLASS_NAME);
    let hwnd = unsafe {
        CreateWindowExW(
            WS_EX_TOOLWINDOW,
            watcher_class_name_wide().as_ptr(),
            title.as_ptr(),
            WS_OVERLAPPED,
            0,
            0,
            0,
            0,
            ptr::null_mut(),
            ptr::null_mut(),
            hinstance,
            ptr::null(),
        )
    };

    if hwnd.is_null() {
        return Err(last_error("CreateWindowExW failed for display watcher"));
    }

    info!("display_watcher: installed, hwnd={:?}", hwnd);
    Ok(())
}

fn watcher_class_name_wide() -> &'static [u16] {
    WATCHER_CLASS_NAME_WIDE
        .get_or_init(|| wide_null(WATCHER_CLASS_NAME))
        .as_slice()
}

fn ensure_watcher_class_registered() -> Result<(), String> {
    if WATCHER_CLASS_ATOM.get().is_some() {
        return Ok(());
    }

    let hinstance = unsafe { GetModuleHandleW(ptr::null()) };
    if hinstance.is_null() {
        return Err(last_error("GetModuleHandleW failed"));
    }

    let class_name = watcher_class_name_wide();
    let class = WNDCLASSW {
        style: 0,
        lpfnWndProc: Some(watcher_wnd_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: hinstance,
        hIcon: ptr::null_mut(),
        hCursor: ptr::null_mut(),
        hbrBackground: ptr::null_mut(),
        lpszMenuName: ptr::null(),
        lpszClassName: class_name.as_ptr(),
    };

    let atom = unsafe { RegisterClassW(&class) };
    if atom == 0 {
        return Err(last_error("RegisterClassW failed"));
    }

    let _ = WATCHER_CLASS_ATOM.set(atom);
    Ok(())
}

unsafe extern "system" fn watcher_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_DISPLAYCHANGE => {
            info!("display_watcher: WM_DISPLAYCHANGE received");
            trigger_refresh();
            0
        }
        WM_DEVICECHANGE if wparam as u32 == DBT_DEVNODES_CHANGED => {
            info!("display_watcher: WM_DEVICECHANGE(DBT_DEVNODES_CHANGED) received");
            trigger_refresh();
            1
        }
        WM_POWERBROADCAST
            if wparam as u32 == PBT_APMRESUMEAUTOMATIC
                || wparam as u32 == PBT_APMRESUMESUSPEND =>
        {
            info!("display_watcher: WM_POWERBROADCAST resume received");
            trigger_refresh();
            1
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

fn trigger_refresh() {
    let (Some(app), Some(state)) = (WATCHER_APP_HANDLE.get(), WATCHER_STATE.get()) else {
        return;
    };

    if let Err(error) = commands::refresh_display_snapshot(app, state) {
        error!(
            "display_watcher: failed to refresh display snapshot: {}",
            error
        );
    }
}

fn wide_null(value: &str) -> Vec<u16> {
    use std::{ffi::OsStr, os::windows::ffi::OsStrExt};

    OsStr::new(value)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

fn last_error(context: &str) -> String {
    format!("{} (win32 error {})", context, unsafe { GetLastError() })
}
