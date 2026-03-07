use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use windows_sys::Win32::UI::WindowsAndMessaging::{GetCursorPos, SetCursorPos};

use crate::state::{DisplayState, NocturnState};

#[derive(Clone)]
struct Rect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

impl Rect {
    fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }

    fn clamp_inside(&self, x: i32, y: i32) -> (i32, i32) {
        let target_x = x.clamp(self.x + 1, self.x + self.width - 2);
        let target_y = y.clamp(self.y + 1, self.y + self.height - 2);
        (target_x, target_y)
    }
}

fn to_rect(display: &DisplayState) -> Rect {
    Rect {
        x: display.x,
        y: display.y,
        width: display.width as i32,
        height: display.height as i32,
    }
}

pub fn sync_cursor_confinement(state: &Arc<Mutex<NocturnState>>) {
    let should_run = {
        let state = state.lock().expect("cursor state poisoned");
        state.blackout_count() > 0
    };

    if should_run {
        start_cursor_loop(state);
    } else {
        stop_cursor_loop(state);
    }
}

fn start_cursor_loop(state: &Arc<Mutex<NocturnState>>) {
    let stop_flag = {
        let mut state = state.lock().expect("cursor state poisoned");

        if state.cursor_stop_flag.is_some() {
            return;
        }

        let stop_flag = Arc::new(AtomicBool::new(false));
        state.cursor_stop_flag = Some(stop_flag.clone());
        stop_flag
    };

    let state = Arc::clone(state);

    thread::spawn(move || {
        while !stop_flag.load(Ordering::Relaxed) {
            let (active_rects, blackout_rects) = {
                let state = state.lock().expect("cursor state poisoned");

                let active_rects = state
                    .displays
                    .values()
                    .filter(|display| !display.is_blacked_out)
                    .map(to_rect)
                    .collect::<Vec<_>>();

                let blackout_rects = state
                    .displays
                    .values()
                    .filter(|display| display.is_blacked_out)
                    .map(to_rect)
                    .collect::<Vec<_>>();

                (active_rects, blackout_rects)
            };

            if active_rects.is_empty() || blackout_rects.is_empty() {
                thread::sleep(Duration::from_millis(16));
                continue;
            }

            let mut point = windows_sys::Win32::Foundation::POINT { x: 0, y: 0 };

            unsafe {
                if GetCursorPos(&mut point) != 0 && blackout_rects.iter().any(|rect| rect.contains(point.x, point.y)) {
                    if let Some(target_rect) = nearest_active_rect(point.x, point.y, &active_rects) {
                        let (target_x, target_y) = target_rect.clamp_inside(point.x, point.y);
                        SetCursorPos(target_x, target_y);
                    }
                }
            }

            thread::sleep(Duration::from_millis(16));
        }
    });
}

fn stop_cursor_loop(state: &Arc<Mutex<NocturnState>>) {
    let mut state = state.lock().expect("cursor state poisoned");
    state.reset_cursor_loop();
}

fn nearest_active_rect(x: i32, y: i32, active_rects: &[Rect]) -> Option<&Rect> {
    active_rects.iter().min_by_key(|rect| distance_to_rect(x, y, rect))
}

fn distance_to_rect(x: i32, y: i32, rect: &Rect) -> i64 {
    let center_x = rect.x + (rect.width / 2);
    let center_y = rect.y + (rect.height / 2);
    let dx = center_x - x;
    let dy = center_y - y;
    (dx as i64 * dx as i64) + (dy as i64 * dy as i64)
}
