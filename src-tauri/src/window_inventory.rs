use std::{collections::HashMap, path::Path, ptr};

use windows_sys::Win32::{
    Foundation::{CloseHandle, HWND, LPARAM, RECT},
    Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED},
    System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
        PROCESS_QUERY_LIMITED_INFORMATION,
    },
    UI::WindowsAndMessaging::{
        EnumWindows, GetForegroundWindow, GetShellWindow, GetWindow, GetWindowLongPtrW,
        GetWindowRect, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId, IsIconic,
        IsWindowVisible, SetForegroundWindow, GWL_EXSTYLE, GW_OWNER, WS_EX_TOOLWINDOW,
    },
};

use crate::state::{DisplayState, HiddenAppSummary};

const MIN_WINDOW_WIDTH: i32 = 120;
const MIN_WINDOW_HEIGHT: i32 = 48;
const NOCTURN_PANEL_TITLE: &str = "Nocturn";
const NOCTURN_OVERLAY_TITLE_PREFIX: &str = "Nocturn Overlay ";

#[derive(Clone)]
struct WindowCandidate {
    app_name: String,
    rect: RECT,
}

pub fn snapshot_hidden_apps_by_display(
    displays: &HashMap<String, DisplayState>,
) -> Result<HashMap<String, Vec<HiddenAppSummary>>, String> {
    let blacked_out_displays = displays
        .values()
        .filter(|display| display.is_blacked_out)
        .cloned()
        .collect::<Vec<_>>();

    if blacked_out_displays.is_empty() {
        return Ok(HashMap::new());
    }

    let windows = enumerate_windows()?;
    let mut grouped = HashMap::<String, HashMap<String, usize>>::new();

    for window in windows {
        let Some(display_id) = display_id_for_window(&window.rect, &blacked_out_displays) else {
            continue;
        };

        *grouped
            .entry(display_id)
            .or_default()
            .entry(window.app_name)
            .or_insert(0) += 1;
    }

    Ok(grouped
        .into_iter()
        .map(|(display_id, apps)| {
            let mut summaries = apps
                .into_iter()
                .map(|(app_name, window_count)| HiddenAppSummary {
                    app_name,
                    window_count,
                })
                .collect::<Vec<_>>();

            summaries.sort_by(|left, right| {
                right
                    .window_count
                    .cmp(&left.window_count)
                    .then_with(|| left.app_name.cmp(&right.app_name))
            });

            (display_id, summaries)
        })
        .collect())
}

fn enumerate_windows() -> Result<Vec<WindowCandidate>, String> {
    let mut windows = Vec::<WindowCandidate>::new();
    let windows_ptr = &mut windows as *mut Vec<WindowCandidate>;
    let enum_result = unsafe { EnumWindows(Some(enum_windows_proc), windows_ptr as LPARAM) };

    if enum_result == 0 {
        return Err("EnumWindows failed.".to_string());
    }

    Ok(windows)
}

unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> i32 {
    let windows = unsafe { &mut *(lparam as *mut Vec<WindowCandidate>) };

    if let Some(window) = inspect_window(hwnd) {
        windows.push(window);
    }

    1
}

fn inspect_window(hwnd: HWND) -> Option<WindowCandidate> {
    if unsafe { IsWindowVisible(hwnd) } == 0 || unsafe { IsIconic(hwnd) } != 0 {
        return None;
    }

    if unsafe { GetWindow(hwnd, GW_OWNER) } != ptr::null_mut() {
        return None;
    }

    let ex_style = unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32 };
    if ex_style & WS_EX_TOOLWINDOW != 0 {
        return None;
    }

    if is_window_cloaked(hwnd) {
        return None;
    }

    let title = read_window_title(hwnd)?;
    if title.is_empty()
        || title == NOCTURN_PANEL_TITLE
        || title.starts_with(NOCTURN_OVERLAY_TITLE_PREFIX)
    {
        return None;
    }

    let rect = read_window_rect(hwnd)?;
    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;
    if width < MIN_WINDOW_WIDTH || height < MIN_WINDOW_HEIGHT {
        return None;
    }

    let app_name = process_app_name(hwnd).unwrap_or_else(|| title.clone());
    if app_name.eq_ignore_ascii_case("nocturn") {
        return None;
    }

    Some(WindowCandidate { app_name, rect })
}

fn read_window_title(hwnd: HWND) -> Option<String> {
    let length = unsafe { GetWindowTextLengthW(hwnd) };
    if length <= 0 {
        return None;
    }

    let mut buffer = vec![0u16; length as usize + 1];
    let copied = unsafe { GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32) };
    if copied <= 0 {
        return None;
    }

    let title = String::from_utf16_lossy(&buffer[..copied as usize]);
    let trimmed = title.trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn read_window_rect(hwnd: HWND) -> Option<RECT> {
    let mut rect = RECT::default();
    let result = unsafe { GetWindowRect(hwnd, &mut rect) };
    if result == 0 {
        None
    } else {
        Some(rect)
    }
}

/// Windows on other virtual desktops and suspended UWP/packaged apps hosted by
/// ApplicationFrameHost stay WS_VISIBLE (and keep their real screen-space
/// rect) while the DWM marks them cloaked. Without this check such windows
/// pass every other visibility test and get misreported as hidden behind an
/// overlay on a monitor they were never actually shown on.
fn is_window_cloaked(hwnd: HWND) -> bool {
    let mut cloaked: u32 = 0;
    let result = unsafe {
        DwmGetWindowAttribute(
            hwnd,
            DWMWA_CLOAKED as u32,
            &mut cloaked as *mut u32 as *mut core::ffi::c_void,
            std::mem::size_of::<u32>() as u32,
        )
    };

    result == 0 && cloaked != 0
}

fn process_app_name(hwnd: HWND) -> Option<String> {
    let mut process_id = 0u32;
    unsafe {
        GetWindowThreadProcessId(hwnd, &mut process_id);
    }

    if process_id == 0 {
        return None;
    }

    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, process_id) };
    if handle.is_null() {
        return None;
    }

    let mut buffer = vec![0u16; 32768];
    let mut length = buffer.len() as u32;
    let result = unsafe {
        QueryFullProcessImageNameW(handle, PROCESS_NAME_WIN32, buffer.as_mut_ptr(), &mut length)
    };
    unsafe {
        CloseHandle(handle);
    }

    if result == 0 || length == 0 {
        return None;
    }

    let full_path = String::from_utf16_lossy(&buffer[..length as usize]);
    let stem = Path::new(&full_path)
        .file_stem()?
        .to_string_lossy()
        .trim()
        .to_string();
    if stem.is_empty() {
        None
    } else {
        Some(stem)
    }
}

/// Minimum share of the window's own area that must overlap a display before
/// the window is considered to be on that display. Guards against a window
/// that merely overhangs a few pixels onto a neighboring monitor being
/// reported as fully hidden there.
const MIN_DISPLAY_OVERLAP_FRACTION: i64 = 4; // 1/4, i.e. 25%

fn rect_area(rect: &RECT) -> i64 {
    i64::from(rect.right - rect.left) * i64::from(rect.bottom - rect.top)
}

fn intersection_area(rect: &RECT, display: &DisplayState) -> i64 {
    let left = rect.left.max(display.x);
    let top = rect.top.max(display.y);
    let right = rect.right.min(display.x + display.width as i32);
    let bottom = rect.bottom.min(display.y + display.height as i32);

    if right <= left || bottom <= top {
        0
    } else {
        i64::from(right - left) * i64::from(bottom - top)
    }
}

/// Assigns a window to whichever display it overlaps the most, rather than
/// only checking the window's center point. A window that straddles two
/// monitors (e.g. mostly on the blacked-out one) is otherwise silently
/// dropped, or attributed to the wrong monitor, by a center-point test.
fn display_id_for_window(rect: &RECT, displays: &[DisplayState]) -> Option<String> {
    let window_area = rect_area(rect);
    if window_area <= 0 {
        return None;
    }

    let (display, area) = displays
        .iter()
        .map(|display| (display, intersection_area(rect, display)))
        .filter(|(_, area)| *area > 0)
        .max_by_key(|(_, area)| *area)?;

    if area * MIN_DISPLAY_OVERLAP_FRACTION < window_area {
        return None;
    }

    Some(display.id.clone())
}

/// The native blackout overlay is WS_EX_NOACTIVATE and never steals focus, so a window
/// that's foreground on the display being blacked out stays foreground (and keeps
/// receiving keystrokes) behind the black screen. If that's the case, hand focus to
/// another eligible top-level window on a surviving display, or to the shell desktop as
/// a last resort, so the user doesn't keep typing invisibly into a hidden window.
pub(crate) fn reassign_foreground_focus_away_from(display: &DisplayState) {
    let foreground = unsafe { GetForegroundWindow() };
    if foreground.is_null() {
        return;
    }

    let Some(foreground_rect) = read_window_rect(foreground) else {
        return;
    };

    if display_id_for_window(&foreground_rect, std::slice::from_ref(display)).is_none() {
        return;
    }

    struct HandoffSearch {
        display: DisplayState,
        exclude: HWND,
        found: Option<HWND>,
    }

    unsafe extern "system" fn candidate_proc(hwnd: HWND, lparam: LPARAM) -> i32 {
        let search = unsafe { &mut *(lparam as *mut HandoffSearch) };

        if hwnd == search.exclude {
            return 1;
        }

        if let Some(candidate) = inspect_focus_candidate(hwnd, &search.display) {
            search.found = Some(candidate);
            return 0;
        }

        1
    }

    let mut search = HandoffSearch {
        display: display.clone(),
        exclude: foreground,
        found: None,
    };
    let search_ptr = &mut search as *mut HandoffSearch;
    unsafe { EnumWindows(Some(candidate_proc), search_ptr as LPARAM) };

    if let Some(target) = search.found {
        unsafe {
            SetForegroundWindow(target);
        }
        return;
    }

    let desktop = unsafe { GetShellWindow() };
    if !desktop.is_null() {
        unsafe {
            SetForegroundWindow(desktop);
        }
    }
}

fn inspect_focus_candidate(hwnd: HWND, display: &DisplayState) -> Option<HWND> {
    if unsafe { IsWindowVisible(hwnd) } == 0 || unsafe { IsIconic(hwnd) } != 0 {
        return None;
    }

    if unsafe { GetWindow(hwnd, GW_OWNER) } != ptr::null_mut() {
        return None;
    }

    let ex_style = unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32 };
    if ex_style & WS_EX_TOOLWINDOW != 0 {
        return None;
    }

    let rect = read_window_rect(hwnd)?;
    if display_id_for_window(&rect, std::slice::from_ref(display)).is_some() {
        return None;
    }

    Some(hwnd)
}
