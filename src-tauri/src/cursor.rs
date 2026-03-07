use std::sync::{Arc, Mutex};

use crate::state::NocturnState;

pub fn sync_cursor_confinement(state: &Arc<Mutex<NocturnState>>) {
    let mut state = state.lock().expect("cursor state poisoned");
    state.reset_cursor_loop();
}
