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
    Foundation::{GetLastError, HWND, LPARAM, LRESULT, RECT, WPARAM},
    Graphics::Gdi::{
        BeginPaint, CreatePen, CreateSolidBrush, DeleteObject, DrawTextW, EndPaint, FillRect,
        GetStockObject, InvalidateRect, PAINTSTRUCT, RoundRect, SelectObject, SetBkMode,
        SetTextColor, BLACK_BRUSH, DT_END_ELLIPSIS, DT_LEFT, DT_NOPREFIX, DT_SINGLELINE,
        DT_VCENTER, HBRUSH, PS_SOLID, TRANSPARENT,
    },
    System::LibraryLoader::GetModuleHandleW,
    UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, GetClientRect, HTCLIENT, HWND_TOPMOST,
        IsWindow, RegisterClassW, SetWindowPos, ShowWindow, SWP_NOACTIVATE, SWP_SHOWWINDOW,
        SW_SHOWNA, WM_NCHITTEST, WM_PAINT, WNDCLASSW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
        WS_EX_TOPMOST, WS_POPUP,
    },
};

use crate::state::{DisplayState, HiddenAppSummary};

pub const OVERLAY_CLASS_NAME: &str = "NocturnNativeOverlay";
const MAX_OVERLAY_CARDS: usize = 5;
const CARD_WIDTH: i32 = 336;
const CARD_HEIGHT: i32 = 34;
const CARD_GAP: i32 = 10;
const CARD_MARGIN: i32 = 24;
const CARD_RADIUS: i32 = 14;

#[derive(Clone, Default)]
struct OverlayWindowRecord {
    hwnd: isize,
    hidden_apps: Vec<HiddenAppSummary>,
}

static OVERLAY_WINDOWS: OnceLock<Mutex<HashMap<String, OverlayWindowRecord>>> = OnceLock::new();
static OVERLAY_CARD_CACHE: OnceLock<Mutex<HashMap<String, Vec<HiddenAppSummary>>>> = OnceLock::new();
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

fn overlay_class_name_wide() -> &'static [u16] {
    OVERLAY_CLASS_NAME_WIDE
        .get_or_init(|| wide_null(OVERLAY_CLASS_NAME))
        .as_slice()
}

fn overlay_windows() -> &'static Mutex<HashMap<String, OverlayWindowRecord>> {
    OVERLAY_WINDOWS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn overlay_card_cache() -> &'static Mutex<HashMap<String, Vec<HiddenAppSummary>>> {
    OVERLAY_CARD_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn sync_overlay_cards(
    app: &AppHandle,
    hidden_apps_by_display: HashMap<String, Vec<HiddenAppSummary>>,
) -> Result<(), String> {
    let app = app.clone();
    thread::spawn(move || {
        if let Err(error) = app.run_on_main_thread(move || {
            update_overlay_cards(hidden_apps_by_display);
        }) {
            error!("sync_overlay_cards: failed to schedule card update: {}", error);
        }
    });

    Ok(())
}

fn create_or_update_overlay(display: &DisplayState) -> Result<(), String> {
    ensure_overlay_class_registered()?;

    let hwnd = {
        let windows = overlay_windows().lock().expect("overlay registry poisoned");
        windows.get(&display.id).map(|record| record.hwnd)
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
        .insert(
            display.id.clone(),
            OverlayWindowRecord {
                hwnd: hwnd as isize,
                hidden_apps: overlay_card_cache()
                    .lock()
                    .expect("overlay card cache poisoned")
                    .get(&display.id)
                    .cloned()
                    .unwrap_or_default(),
            },
        );

    info!(
        "create_or_update_overlay: created native overlay for {}",
        display.id
    );
    Ok(())
}

fn destroy_overlay(display_id: &str) -> Result<(), String> {
    let record = overlay_windows()
        .lock()
        .expect("overlay registry poisoned")
        .remove(display_id);

    let Some(record) = record else {
        return Ok(());
    };

    let hwnd = record.hwnd as HWND;
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
        WM_PAINT => {
            unsafe { paint_overlay(hwnd) };
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

unsafe fn paint_overlay(hwnd: HWND) {
    let mut paint = PAINTSTRUCT::default();
    let hdc = unsafe { BeginPaint(hwnd, &mut paint) };
    if hdc.is_null() {
        return;
    }

    let mut client_rect = RECT::default();
    unsafe {
        GetClientRect(hwnd, &mut client_rect);
        FillRect(hdc, &client_rect, GetStockObject(BLACK_BRUSH) as HBRUSH);
    }

    let hidden_apps = hidden_apps_for_hwnd(hwnd);
    let card_lines = overlay_card_lines(&hidden_apps);

    if !card_lines.is_empty() {
        unsafe {
            SetBkMode(hdc, TRANSPARENT as i32);
        }

        let card_brush = unsafe { CreateSolidBrush(rgb(19, 19, 26)) };
        let card_pen = unsafe { CreatePen(PS_SOLID, 1, rgb(63, 63, 86)) };
        let old_brush = unsafe { SelectObject(hdc, card_brush as _) };
        let old_pen = unsafe { SelectObject(hdc, card_pen as _) };

        for (index, line) in card_lines.iter().enumerate() {
            let top = CARD_MARGIN + index as i32 * (CARD_HEIGHT + CARD_GAP);
            let mut card_rect = RECT {
                left: CARD_MARGIN,
                top,
                right: (CARD_MARGIN + CARD_WIDTH).min(client_rect.right - CARD_MARGIN),
                bottom: top + CARD_HEIGHT,
            };

            unsafe {
                RoundRect(
                    hdc,
                    card_rect.left,
                    card_rect.top,
                    card_rect.right,
                    card_rect.bottom,
                    CARD_RADIUS,
                    CARD_RADIUS,
                );
                SetTextColor(hdc, rgb(241, 245, 249));
            }

            card_rect.left += 12;
            card_rect.right -= 12;
            unsafe {
                DrawTextW(
                    hdc,
                    wide_null(line).as_ptr(),
                    -1,
                    &mut card_rect,
                    DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS | DT_NOPREFIX,
                );
            }
        }

        unsafe {
            SelectObject(hdc, old_brush);
            SelectObject(hdc, old_pen);
            DeleteObject(card_brush as _);
            DeleteObject(card_pen as _);
        }
    }

    unsafe {
        EndPaint(hwnd, &paint);
    }
}

fn hidden_apps_for_hwnd(hwnd: HWND) -> Vec<HiddenAppSummary> {
    overlay_windows()
        .lock()
        .expect("overlay registry poisoned")
        .values()
        .find(|record| record.hwnd == hwnd as isize)
        .map(|record| record.hidden_apps.clone())
        .unwrap_or_default()
}

fn overlay_card_lines(hidden_apps: &[HiddenAppSummary]) -> Vec<String> {
    if hidden_apps.is_empty() {
        return vec!["No visible apps detected".to_string()];
    }

    let mut lines = hidden_apps
        .iter()
        .take(MAX_OVERLAY_CARDS)
        .map(|app| {
            if app.window_count > 1 {
                format!("{}  |  {} windows", app.app_name, app.window_count)
            } else {
                app.app_name.clone()
            }
        })
        .collect::<Vec<_>>();

    if hidden_apps.len() > MAX_OVERLAY_CARDS {
        lines.push(format!("+{} more apps", hidden_apps.len() - MAX_OVERLAY_CARDS));
    }

    lines
}

fn update_overlay_cards(hidden_apps_by_display: HashMap<String, Vec<HiddenAppSummary>>) {
    {
        let mut cache = overlay_card_cache()
            .lock()
            .expect("overlay card cache poisoned");
        *cache = hidden_apps_by_display.clone();
    }

    let mut windows = overlay_windows().lock().expect("overlay registry poisoned");

    for (display_id, record) in windows.iter_mut() {
        record.hidden_apps = hidden_apps_by_display
            .get(display_id)
            .cloned()
            .unwrap_or_default();

        let hwnd = record.hwnd as HWND;
        if unsafe { IsWindow(hwnd) } != 0 {
            unsafe {
                InvalidateRect(hwnd, ptr::null(), 1);
            }
        }
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

fn rgb(red: u8, green: u8, blue: u8) -> u32 {
    red as u32 | ((green as u32) << 8) | ((blue as u32) << 16)
}
