use std::sync::{Arc, Mutex};

use windows_sys::Win32::{
    Foundation::{POINT, RECT},
    UI::WindowsAndMessaging::{ClipCursor, GetCursorPos, SetCursorPos},
};

use crate::state::{DisplayState, NocturnState};

pub fn sync_cursor_confinement(state: &Arc<Mutex<NocturnState>>) {
    let (should_confine, bounds) = {
        let state = state.lock().expect("cursor state poisoned");
        let active_displays = state
            .displays
            .values()
            .filter(|display| !display.is_blacked_out)
            .collect::<Vec<_>>();

        (
            !state.settings.allow_cursor_exit_active_displays && !active_displays.is_empty(),
            display_bounds(&active_displays),
        )
    };

    if should_confine {
        if let Some(bounds) = bounds {
            confine_cursor(bounds);
            return;
        }
    }

    release_cursor();
}

fn display_bounds(displays: &[&DisplayState]) -> Option<RECT> {
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

fn release_cursor() {
    unsafe {
        ClipCursor(std::ptr::null());
    }
}
