use std::sync::{Arc, Mutex};

use windows_sys::Win32::{
    Foundation::{POINT, RECT},
    UI::WindowsAndMessaging::{ClipCursor, GetCursorPos, SetCursorPos},
};

use crate::state::{DisplayState, NocturnState};

pub fn sync_cursor_confinement(state: &Arc<Mutex<NocturnState>>) {
    let (should_confine, active_displays, blacked_out_displays) = {
        let state = state.lock().expect("cursor state poisoned");
        let active_displays = state
            .displays
            .values()
            .filter(|display| !display.is_blacked_out)
            .cloned()
            .collect::<Vec<_>>();
        let blacked_out_displays = state
            .displays
            .values()
            .filter(|display| display.is_blacked_out)
            .cloned()
            .collect::<Vec<_>>();

        (
            !state.settings.allow_cursor_exit_active_displays && !active_displays.is_empty(),
            active_displays,
            blacked_out_displays,
        )
    };

    if should_confine {
        if let Some(bounds) = confinement_bounds(&active_displays, &blacked_out_displays) {
            confine_cursor(bounds);
            return;
        }
    }

    release_cursor();
}

/// ClipCursor only accepts a single rectangle, so the union bounding box of the
/// active displays is not a safe clip whenever a blacked-out display sits
/// inside that box (e.g. the middle monitor of a 3-in-a-row layout) — the
/// cursor could still land on it. When that happens, fall back to clipping to
/// whichever active display currently contains the cursor instead, so the
/// cursor can never rest on the blacked-out one. This doesn't let the cursor
/// roam freely across all active displays in that layout (Win32 has no way to
/// express a non-rectangular clip region without a mouse hook), but it closes
/// the hole where the cursor could disappear onto a black screen.
fn confinement_bounds(
    active_displays: &[DisplayState],
    blacked_out_displays: &[DisplayState],
) -> Option<RECT> {
    let bounds = display_bounds(active_displays)?;

    let bounds_enclose_blacked_out_display = blacked_out_displays
        .iter()
        .any(|display| display_is_inside(display, &bounds));

    if !bounds_enclose_blacked_out_display {
        return Some(bounds);
    }

    Some(current_display_bounds(active_displays).unwrap_or(bounds))
}

fn display_is_inside(display: &DisplayState, bounds: &RECT) -> bool {
    display.x >= bounds.left
        && display.y >= bounds.top
        && display.x + display.width as i32 <= bounds.right
        && display.y + display.height as i32 <= bounds.bottom
}

/// The active display currently containing the cursor, as a clip rectangle. If
/// the cursor isn't over any active display (e.g. it's already sitting on the
/// display that was just blacked out), falls back to the nearest active
/// display by center distance so the clip still pulls the cursor toward an
/// active screen instead of leaving it clipped to the whole (unsafe) union box.
fn current_display_bounds(active_displays: &[DisplayState]) -> Option<RECT> {
    let mut point = POINT { x: 0, y: 0 };
    if unsafe { GetCursorPos(&mut point) } == 0 {
        return None;
    }

    let containing = active_displays
        .iter()
        .find(|display| point_in_display(&point, display));

    containing
        .or_else(|| nearest_display(active_displays, &point))
        .map(display_rect)
}

fn point_in_display(point: &POINT, display: &DisplayState) -> bool {
    point.x >= display.x
        && point.x < display.x + display.width as i32
        && point.y >= display.y
        && point.y < display.y + display.height as i32
}

fn nearest_display<'a>(displays: &'a [DisplayState], point: &POINT) -> Option<&'a DisplayState> {
    displays.iter().min_by_key(|display| {
        let center_x = display.x + display.width as i32 / 2;
        let center_y = display.y + display.height as i32 / 2;
        let dx = (center_x - point.x) as i64;
        let dy = (center_y - point.y) as i64;
        dx * dx + dy * dy
    })
}

fn display_rect(display: &DisplayState) -> RECT {
    RECT {
        left: display.x,
        top: display.y,
        right: display.x + display.width as i32,
        bottom: display.y + display.height as i32,
    }
}

fn display_bounds(displays: &[DisplayState]) -> Option<RECT> {
    let first = displays.first()?;

    let mut left = first.x;
    let mut top = first.y;
    let mut right = first.x + first.width as i32;
    let mut bottom = first.y + first.height as i32;

    for display in displays.iter().skip(1) {
        left = left.min(display.x);
        top = top.min(display.y);
        right = right.max(display.x + display.width as i32);
        bottom = bottom.max(display.y + display.height as i32);
    }

    Some(RECT {
        left,
        top,
        right,
        bottom,
    })
}

fn confine_cursor(bounds: RECT) {
    unsafe {
        ClipCursor(&bounds);

        let mut point = POINT { x: 0, y: 0 };
        if GetCursorPos(&mut point) != 0 {
            let clamped_x = point.x.clamp(bounds.left, bounds.right.saturating_sub(1));
            let clamped_y = point.y.clamp(bounds.top, bounds.bottom.saturating_sub(1));

            if clamped_x != point.x || clamped_y != point.y {
                SetCursorPos(clamped_x, clamped_y);
            }
        }
    }
}

/// Releases the cursor clip (`ClipCursor(NULL)`). Safe to call even when
/// nothing is currently clipped. Exposed so app-exit paths can release the
/// clip before the process goes away, since Win32 doesn't reliably auto-clear
/// it for a `WS_EX_NOACTIVATE` app that never becomes the foreground window.
pub fn release_cursor() {
    unsafe {
        ClipCursor(std::ptr::null());
    }
}
