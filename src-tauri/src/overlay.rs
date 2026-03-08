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
        BeginPaint, CLIP_DEFAULT_PRECIS, CLEARTYPE_QUALITY, CreateFontIndirectW, CreatePen,
        CreateSolidBrush, DEFAULT_CHARSET, DEFAULT_PITCH, DeleteObject, DrawTextW, EndPaint,
        FF_DONTCARE, FillRect, FW_MEDIUM, FW_SEMIBOLD, GetStockObject, HDC, InvalidateRect,
        LF_FACESIZE, LOGFONTW, NULL_PEN, OUT_DEFAULT_PRECIS, PAINTSTRUCT, RoundRect,
        SelectObject, SetBkMode, SetTextColor, ANTIALIASED_QUALITY, BLACK_BRUSH,
        DT_END_ELLIPSIS, DT_LEFT, DT_NOPREFIX, DT_SINGLELINE, DT_VCENTER, HBRUSH, PS_SOLID,
        TRANSPARENT,
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
const MAX_OVERLAY_ROWS: usize = 4;
const LABEL_STACK_WIDTH: i32 = 248;
const LABEL_STACK_MARGIN: i32 = 22;
const LABEL_STACK_GAP: i32 = 8;
const LABEL_PILL_HEIGHT: i32 = 24;
const LABEL_PILL_RADIUS: i32 = 12;
const ANCHOR_LENGTH: i32 = 18;
const ANCHOR_GAP: i32 = 8;

#[derive(Clone, Copy, Default)]
pub enum OverlayDock {
    Top,
    Right,
    #[default]
    Bottom,
    Left,
    Center,
}

#[derive(Clone, Default)]
pub struct OverlayPresentation {
    pub hidden_apps: Vec<HiddenAppSummary>,
    pub dock: OverlayDock,
    pub is_enabled: bool,
}

#[derive(Clone, Default)]
struct OverlayWindowRecord {
    hwnd: isize,
    presentation: OverlayPresentation,
}

static OVERLAY_WINDOWS: OnceLock<Mutex<HashMap<String, OverlayWindowRecord>>> = OnceLock::new();
static OVERLAY_CARD_CACHE: OnceLock<Mutex<HashMap<String, OverlayPresentation>>> = OnceLock::new();
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

fn overlay_card_cache() -> &'static Mutex<HashMap<String, OverlayPresentation>> {
    OVERLAY_CARD_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn sync_overlay_cards(
    app: &AppHandle,
    overlay_presentations: HashMap<String, OverlayPresentation>,
) -> Result<(), String> {
    let app = app.clone();
    thread::spawn(move || {
        if let Err(error) = app.run_on_main_thread(move || {
            update_overlay_cards(overlay_presentations);
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
                presentation: overlay_card_cache()
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

    let presentation = presentation_for_hwnd(hwnd);
    if !presentation.is_enabled {
        unsafe {
            EndPaint(hwnd, &paint);
        }
        return;
    }
    let rows = overlay_rows(&presentation.hidden_apps);
    let label_stack_rect =
        overlay_label_stack_rect(&client_rect, &presentation.dock, rows.len() as i32);

    if !rows.is_empty() {
        unsafe {
            SetBkMode(hdc, TRANSPARENT as i32);
        }

        draw_overlay_labels(hdc, label_stack_rect, &presentation.dock, &rows);
    }

    unsafe {
        EndPaint(hwnd, &paint);
    }
}

fn presentation_for_hwnd(hwnd: HWND) -> OverlayPresentation {
    overlay_windows()
        .lock()
        .expect("overlay registry poisoned")
        .values()
        .find(|record| record.hwnd == hwnd as isize)
        .map(|record| record.presentation.clone())
        .unwrap_or_default()
}

fn overlay_rows(hidden_apps: &[HiddenAppSummary]) -> Vec<OverlayRow> {
    if hidden_apps.is_empty() {
        return vec![OverlayRow {
            label: "No visible apps detected".to_string(),
            badge: None,
            subdued: true,
        }];
    }

    let mut rows = hidden_apps
        .iter()
        .take(MAX_OVERLAY_ROWS)
        .map(|app| {
            let badge = if app.window_count > 1 {
                Some(format!("{} windows", app.window_count))
            } else {
                None
            };

            OverlayRow {
                label: app.app_name.clone(),
                badge,
                subdued: false,
            }
        })
        .collect::<Vec<_>>();

    if hidden_apps.len() > MAX_OVERLAY_ROWS {
        rows.push(OverlayRow {
            label: format!("+{} more apps", hidden_apps.len() - MAX_OVERLAY_ROWS),
            badge: None,
            subdued: true,
        });
    }

    rows
}

fn update_overlay_cards(overlay_presentations: HashMap<String, OverlayPresentation>) {
    {
        let mut cache = overlay_card_cache()
            .lock()
            .expect("overlay card cache poisoned");
        *cache = overlay_presentations.clone();
    }

    let mut windows = overlay_windows().lock().expect("overlay registry poisoned");

    for (display_id, record) in windows.iter_mut() {
        record.presentation = overlay_presentations
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

#[derive(Clone)]
struct OverlayRow {
    label: String,
    badge: Option<String>,
    subdued: bool,
}

fn overlay_label_stack_rect(client_rect: &RECT, dock: &OverlayDock, row_count: i32) -> RECT {
    let row_count = row_count.max(1);
    let width = LABEL_STACK_WIDTH.min((client_rect.right - client_rect.left) - LABEL_STACK_MARGIN * 2);
    let height = row_count * LABEL_PILL_HEIGHT + (row_count - 1) * LABEL_STACK_GAP;

    let horizontal_center = ((client_rect.right - width) / 2).max(LABEL_STACK_MARGIN);
    let vertical_center = ((client_rect.bottom - height) / 2).max(LABEL_STACK_MARGIN);

    match dock {
        OverlayDock::Top => rect(horizontal_center, LABEL_STACK_MARGIN, width, height),
        OverlayDock::Bottom => rect(
            horizontal_center,
            client_rect.bottom - LABEL_STACK_MARGIN - height,
            width,
            height,
        ),
        OverlayDock::Left => rect(LABEL_STACK_MARGIN, vertical_center, width, height),
        OverlayDock::Right => rect(
            client_rect.right - LABEL_STACK_MARGIN - width,
            vertical_center,
            width,
            height,
        ),
        OverlayDock::Center => rect(
            horizontal_center,
            client_rect.bottom - LABEL_STACK_MARGIN - height,
            width,
            height,
        ),
    }
}

fn draw_overlay_labels(hdc: HDC, stack_rect: RECT, dock: &OverlayDock, rows: &[OverlayRow]) {
    unsafe {
        let pill_brush = CreateSolidBrush(rgb(13, 15, 19));
        let pill_border_pen = CreatePen(PS_SOLID, 1, rgb(28, 32, 39));
        let anchor_pen = CreatePen(PS_SOLID, 1, rgb(72, 79, 91));
        let old_pill_brush = SelectObject(hdc, pill_brush as _);
        let old_pill_pen = SelectObject(hdc, pill_border_pen as _);

        let body_font = create_overlay_font(-15, FW_MEDIUM, CLEARTYPE_QUALITY, "Segoe UI");
        let meta_font = create_overlay_font(-11, FW_MEDIUM, ANTIALIASED_QUALITY, "Segoe UI");

        let old_font = SelectObject(hdc, body_font as _);
        draw_overlay_anchor(hdc, stack_rect, dock, anchor_pen as isize);

        for (index, row) in rows.iter().enumerate() {
            let row_top = stack_rect.top + index as i32 * (LABEL_PILL_HEIGHT + LABEL_STACK_GAP);
            let row_rect = rect(
                stack_rect.left,
                row_top,
                stack_rect.right - stack_rect.left,
                LABEL_PILL_HEIGHT,
            );

            RoundRect(
                hdc,
                row_rect.left,
                row_rect.top,
                row_rect.right,
                row_rect.bottom,
                LABEL_PILL_RADIUS,
                LABEL_PILL_RADIUS,
            );

            let dot_brush = CreateSolidBrush(if row.subdued {
                rgb(86, 94, 106)
            } else {
                rgb(132, 142, 156)
            });
            let old_dot_brush = SelectObject(hdc, dot_brush as _);
            let old_dot_pen = SelectObject(hdc, GetStockObject(NULL_PEN) as _);
            RoundRect(
                hdc,
                row_rect.left + 10,
                row_rect.top + 9,
                row_rect.left + 14,
                row_rect.top + 13,
                6,
                6,
            );
            SelectObject(hdc, old_dot_brush);
            SelectObject(hdc, old_dot_pen);
            DeleteObject(dot_brush as _);

            SetTextColor(
                hdc,
                if row.subdued {
                    rgb(123, 132, 145)
                } else {
                    rgb(221, 227, 236)
                },
            );
            let reserved_badge_width = if row.badge.is_some() { 42 } else { 14 };
            let mut label_rect = RECT {
                left: row_rect.left + 22,
                top: row_rect.top,
                right: row_rect.right - reserved_badge_width,
                bottom: row_rect.bottom,
            };
            DrawTextW(
                hdc,
                wide_null(&row.label).as_ptr(),
                -1,
                &mut label_rect,
                DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS | DT_NOPREFIX,
            );

            if let Some(badge) = &row.badge {
                SelectObject(hdc, meta_font as _);
                SetTextColor(hdc, rgb(114, 124, 137));
                let mut badge_text_rect = RECT {
                    left: row_rect.right - 34,
                    top: row_rect.top,
                    right: row_rect.right - 10,
                    bottom: row_rect.bottom,
                };
                DrawTextW(
                    hdc,
                    wide_null(badge).as_ptr(),
                    -1,
                    &mut badge_text_rect,
                    DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS | DT_NOPREFIX,
                );
                SelectObject(hdc, body_font as _);
            }
        }

        SelectObject(hdc, old_font);
        SelectObject(hdc, old_pill_brush);
        SelectObject(hdc, old_pill_pen);

        DeleteObject(body_font as _);
        DeleteObject(meta_font as _);
        DeleteObject(pill_brush as _);
        DeleteObject(pill_border_pen as _);
        DeleteObject(anchor_pen as _);
    }
}

fn create_overlay_font(height: i32, weight: u32, quality: u8, face_name: &str) -> isize {
    let mut logfont = LOGFONTW {
        lfHeight: height,
        lfWeight: weight as i32,
        lfCharSet: DEFAULT_CHARSET,
        lfOutPrecision: OUT_DEFAULT_PRECIS,
        lfClipPrecision: CLIP_DEFAULT_PRECIS,
        lfQuality: quality,
        lfPitchAndFamily: DEFAULT_PITCH | FF_DONTCARE,
        ..Default::default()
    };

    let face_name_wide = wide_null(face_name);
    let limit = face_name_wide.len().min(LF_FACESIZE as usize);
    logfont.lfFaceName[..limit].copy_from_slice(&face_name_wide[..limit]);

    unsafe { CreateFontIndirectW(&logfont) as isize }
}

fn rect(left: i32, top: i32, width: i32, height: i32) -> RECT {
    RECT {
        left,
        top,
        right: left + width.max(0),
        bottom: top + height.max(0),
    }
}

fn draw_overlay_anchor(hdc: HDC, stack_rect: RECT, dock: &OverlayDock, anchor_pen: isize) {
    unsafe {
        let old_pen = SelectObject(hdc, anchor_pen as _);
        let old_brush = SelectObject(hdc, GetStockObject(NULL_PEN) as _);
        let anchor_brush = CreateSolidBrush(rgb(126, 137, 154));
        let old_anchor_brush = SelectObject(hdc, anchor_brush as _);

        match dock {
            OverlayDock::Top => {
                RoundRect(
                    hdc,
                    stack_rect.left + 18,
                    stack_rect.top - ANCHOR_GAP - 2,
                    stack_rect.left + 18 + ANCHOR_LENGTH,
                    stack_rect.top - ANCHOR_GAP,
                    2,
                    2,
                );
                RoundRect(
                    hdc,
                    stack_rect.left + 12,
                    stack_rect.top - ANCHOR_GAP - 4,
                    stack_rect.left + 18,
                    stack_rect.top - ANCHOR_GAP + 2,
                    6,
                    6,
                );
            }
            OverlayDock::Bottom => {
                RoundRect(
                    hdc,
                    stack_rect.left + 18,
                    stack_rect.bottom + ANCHOR_GAP,
                    stack_rect.left + 18 + ANCHOR_LENGTH,
                    stack_rect.bottom + ANCHOR_GAP + 2,
                    2,
                    2,
                );
                RoundRect(
                    hdc,
                    stack_rect.left + 12,
                    stack_rect.bottom + ANCHOR_GAP - 2,
                    stack_rect.left + 18,
                    stack_rect.bottom + ANCHOR_GAP + 4,
                    6,
                    6,
                );
            }
            OverlayDock::Left => {
                RoundRect(
                    hdc,
                    stack_rect.left - ANCHOR_GAP - 2,
                    stack_rect.top + 18,
                    stack_rect.left - ANCHOR_GAP,
                    stack_rect.top + 18 + ANCHOR_LENGTH,
                    2,
                    2,
                );
                RoundRect(
                    hdc,
                    stack_rect.left - ANCHOR_GAP - 4,
                    stack_rect.top + 12,
                    stack_rect.left - ANCHOR_GAP + 2,
                    stack_rect.top + 18,
                    6,
                    6,
                );
            }
            OverlayDock::Right => {
                RoundRect(
                    hdc,
                    stack_rect.right + ANCHOR_GAP,
                    stack_rect.top + 18,
                    stack_rect.right + ANCHOR_GAP + 2,
                    stack_rect.top + 18 + ANCHOR_LENGTH,
                    2,
                    2,
                );
                RoundRect(
                    hdc,
                    stack_rect.right + ANCHOR_GAP - 2,
                    stack_rect.top + 12,
                    stack_rect.right + ANCHOR_GAP + 4,
                    stack_rect.top + 18,
                    6,
                    6,
                );
            }
            OverlayDock::Center => {}
        }

        SelectObject(hdc, old_anchor_brush);
        SelectObject(hdc, old_brush);
        SelectObject(hdc, old_pen);
        DeleteObject(anchor_brush as _);
    }
}
