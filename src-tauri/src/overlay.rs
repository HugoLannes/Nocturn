use std::{
    collections::{HashMap, HashSet},
    ffi::OsStr,
    os::windows::ffi::OsStrExt,
    ptr,
    sync::{Mutex, OnceLock},
    thread,
};

use log::{error, info};
use serde::Serialize;
use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, Position, Size, WebviewUrl,
    WebviewWindow, WebviewWindowBuilder,
};
use windows_sys::Win32::{
    Foundation::{GetLastError, HWND, LPARAM, LRESULT, RECT, WPARAM},
    Graphics::Gdi::{BeginPaint, EndPaint, FillRect, GetStockObject, PAINTSTRUCT, BLACK_BRUSH},
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
const MAX_OVERLAY_ROWS: usize = 4;
const CARD_WIDTH: i32 = 304;
const CARD_MARGIN: i32 = 18;
const CARD_GAP: i32 = 7;
const CARD_ROW_HEIGHT: i32 = 42;
const CARD_PADDING_Y: i32 = 12;
const CARD_WINDOW_PADDING: i32 = 0;

#[derive(Clone, Copy, Debug, Default, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OverlayDock {
    Top,
    Right,
    #[default]
    Bottom,
    Left,
    Center,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OverlayPresentation {
    pub hidden_apps: Vec<HiddenAppSummary>,
    pub dock: OverlayDock,
    pub is_enabled: bool,
}

#[derive(Clone, Copy, Default)]
struct OverlayWindowRecord {
    hwnd: isize,
}

static OVERLAY_WINDOWS: OnceLock<Mutex<HashMap<String, OverlayWindowRecord>>> = OnceLock::new();
static OVERLAY_CARD_CACHE: OnceLock<Mutex<HashMap<String, OverlayPresentation>>> = OnceLock::new();
static OVERLAY_CLASS_ATOM: OnceLock<u16> = OnceLock::new();
static OVERLAY_CLASS_NAME_WIDE: OnceLock<Vec<u16>> = OnceLock::new();

pub fn overlay_label(display_id: &str) -> String {
    format!("overlay-{}", sanitize_label_component(display_id))
}

pub fn get_overlay_card_presentation(window_label: &str) -> Option<OverlayPresentation> {
    overlay_card_cache()
        .lock()
        .expect("overlay card cache poisoned")
        .get(window_label)
        .cloned()
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
        }
    });

    Ok(())
}

pub fn close_overlay(app: &AppHandle, display_id: &str) -> Result<(), String> {
    let native_label = overlay_label(display_id);
    let card_label = overlay_card_label(display_id);
    let display_id = display_id.to_string();
    let app = app.clone();

    info!("close_overlay: {}", native_label);

    thread::spawn(move || {
        let native_log_label = native_label.clone();
        let card_log_label = card_label.clone();
        let app_handle = app.clone();

        if let Err(error) = app.run_on_main_thread(move || {
            if let Err(error) = destroy_overlay(&display_id) {
                error!(
                    "close_overlay: failed to destroy native overlay {}: {}",
                    native_log_label, error
                );
            }

            if let Err(error) = destroy_overlay_card_window(&app_handle, &card_label) {
                error!(
                    "close_overlay: failed to destroy overlay card {}: {}",
                    card_log_label, error
                );
            }
        }) {
            error!(
                "close_overlay: failed to schedule destroy for {}: {}",
                native_label, error
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

pub fn sync_overlay_cards(
    app: &AppHandle,
    displays: &HashMap<String, DisplayState>,
    overlay_presentations: HashMap<String, OverlayPresentation>,
) -> Result<(), String> {
    let app = app.clone();
    let displays = displays.clone();

    thread::spawn(move || {
        let app_handle = app.clone();

        if let Err(error) = app.run_on_main_thread(move || {
            if let Err(error) =
                sync_overlay_cards_on_main_thread(&app_handle, &displays, overlay_presentations)
            {
                error!("sync_overlay_cards: {}", error);
            }
        }) {
            error!("sync_overlay_cards: failed to schedule card update: {}", error);
        }
    });

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

fn overlay_card_cache() -> &'static Mutex<HashMap<String, OverlayPresentation>> {
    OVERLAY_CARD_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn overlay_card_label(display_id: &str) -> String {
    format!("overlay-card-{}", sanitize_label_component(display_id))
}

fn sanitize_label_component(value: &str) -> String {
    value
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
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
        .insert(display.id.clone(), OverlayWindowRecord { hwnd: hwnd as isize });

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
        hbrBackground: unsafe { GetStockObject(BLACK_BRUSH) as _ },
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
        FillRect(hdc, &client_rect, GetStockObject(BLACK_BRUSH) as _);
        EndPaint(hwnd, &paint);
    }
}

fn sync_overlay_cards_on_main_thread(
    app: &AppHandle,
    displays: &HashMap<String, DisplayState>,
    overlay_presentations: HashMap<String, OverlayPresentation>,
) -> Result<(), String> {
    let active_labels = overlay_presentations
        .iter()
        .filter(|(_, presentation)| presentation.is_enabled)
        .map(|(display_id, _)| overlay_card_label(display_id))
        .collect::<HashSet<_>>();

    let stale_labels = {
        let mut cache = overlay_card_cache()
            .lock()
            .expect("overlay card cache poisoned");
        let stale = cache
            .keys()
            .filter(|label| !active_labels.contains(*label))
            .cloned()
            .collect::<Vec<_>>();

        cache.retain(|label, _| active_labels.contains(label));

        for (display_id, presentation) in &overlay_presentations {
            if presentation.is_enabled {
                cache.insert(overlay_card_label(display_id), presentation.clone());
            }
        }

        stale
    };

    for stale_label in stale_labels {
        destroy_overlay_card_window(app, &stale_label)?;
    }

    for (display_id, presentation) in overlay_presentations {
        let card_label = overlay_card_label(&display_id);

        if !presentation.is_enabled {
            destroy_overlay_card_window(app, &card_label)?;
            continue;
        }

        let Some(display) = displays.get(&display_id) else {
            destroy_overlay_card_window(app, &card_label)?;
            continue;
        };

        let window = create_or_update_overlay_card_window(app, display, &card_label, &presentation)?;
        window
            .emit("overlay-card:update", presentation)
            .map_err(|error| error.to_string())?;
    }

    Ok(())
}

fn create_or_update_overlay_card_window(
    app: &AppHandle,
    display: &DisplayState,
    card_label: &str,
    presentation: &OverlayPresentation,
) -> Result<WebviewWindow, String> {
    let card_rect = overlay_card_rect(display, presentation);
    let width = (card_rect.right - card_rect.left) as u32;
    let height = (card_rect.bottom - card_rect.top) as u32;
    let position = Position::Physical(PhysicalPosition::new(card_rect.left, card_rect.top));
    let size = Size::Physical(PhysicalSize::new(width, height));

    if let Some(window) = app.get_webview_window(card_label) {
        window.set_position(position).map_err(|error| error.to_string())?;
        window.set_size(size).map_err(|error| error.to_string())?;
        window.show().map_err(|error| error.to_string())?;
        return Ok(window);
    }

    let window = WebviewWindowBuilder::new(app, card_label.to_string(), WebviewUrl::App("index.html".into()))
        .transparent(true)
        .decorations(false)
        .resizable(false)
        .skip_taskbar(true)
        .always_on_top(true)
        .visible(true)
        .inner_size(width as f64, height as f64)
        .position(card_rect.left as f64, card_rect.top as f64)
        .build()
        .map_err(|error| error.to_string())?;

    window.show().map_err(|error| error.to_string())?;

    Ok(window)
}

fn destroy_overlay_card_window(app: &AppHandle, card_label: &str) -> Result<(), String> {
    overlay_card_cache()
        .lock()
        .expect("overlay card cache poisoned")
        .remove(card_label);

    let Some(window) = app.get_webview_window(card_label) else {
        return Ok(());
    };

    window.close().map_err(|error| error.to_string())
}

fn overlay_card_rect(display: &DisplayState, presentation: &OverlayPresentation) -> RECT {
    let client_rect = rect(0, 0, display.width as i32, display.height as i32);
    let row_count = overlay_row_count(&presentation.hidden_apps) as i32;
    let card_inner_height =
        CARD_PADDING_Y * 2 + row_count * CARD_ROW_HEIGHT + row_count.saturating_sub(1) * CARD_GAP;
    let outer_width = CARD_WIDTH + CARD_WINDOW_PADDING * 2;
    let outer_height = card_inner_height + CARD_WINDOW_PADDING * 2;

    let local_rect = overlay_card_local_rect(&client_rect, &presentation.dock, outer_width, outer_height);

    rect(
        display.x + local_rect.left,
        display.y + local_rect.top,
        outer_width,
        outer_height,
    )
}

fn overlay_card_local_rect(client_rect: &RECT, dock: &OverlayDock, width: i32, height: i32) -> RECT {
    let horizontal_center = ((client_rect.right - width) / 2).max(CARD_MARGIN);
    let vertical_center = ((client_rect.bottom - height) / 2).max(CARD_MARGIN);

    match dock {
        OverlayDock::Top => rect(horizontal_center, CARD_MARGIN, width, height),
        OverlayDock::Bottom => rect(
            horizontal_center,
            client_rect.bottom - CARD_MARGIN - height,
            width,
            height,
        ),
        OverlayDock::Left => rect(CARD_MARGIN, vertical_center, width, height),
        OverlayDock::Right => rect(
            client_rect.right - CARD_MARGIN - width,
            vertical_center,
            width,
            height,
        ),
        OverlayDock::Center => rect(horizontal_center, vertical_center, width, height),
    }
}

fn overlay_row_count(hidden_apps: &[HiddenAppSummary]) -> usize {
    if hidden_apps.is_empty() {
        1
    } else {
        hidden_apps.len().min(MAX_OVERLAY_ROWS) + usize::from(hidden_apps.len() > MAX_OVERLAY_ROWS)
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

fn rect(left: i32, top: i32, width: i32, height: i32) -> RECT {
    RECT {
        left,
        top,
        right: left + width.max(0),
        bottom: top + height.max(0),
    }
}
