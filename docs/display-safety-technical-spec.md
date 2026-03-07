# Nocturn - Display Safety Technical Spec

**Version:** v0.2  
**Status:** Implemented reference  
**Scope:** Current desktop behavior for panel safety, blackout overlays, and state flow

---

## 1. Purpose

Nocturn must let the user black out displays without losing access to the control panel or leaving the app in an unusable state.

The current system enforces two core invariants:

1. at least one display must remain active
2. the panel must not stay on a display that is about to be blacked out

---

## 2. Current Product Rules

### 2.1 `n-1` Safety Rule

If `n` displays are connected, at most `n-1` can be blacked out.

- `1` display -> blackout is refused
- `2` displays -> at most `1` can be blacked out
- `3` displays -> at most `2` can be blacked out

The backend is the source of truth for this rule.

### 2.2 Panel Reachability Rule

If the visible panel is on the targeted display, the panel is moved to another active display before the blackout is applied.

### 2.3 Wake Rule

All blacked-out displays can be restored by:

- clicking `Wake all displays` in the panel
- double-pressing `Space` while at least one display is blacked out

### 2.4 Cursor Behavior

Cursor confinement is currently disabled.

Earlier confinement logic caused the cursor to feel trapped and is not part of the current behavior.

---

## 3. Runtime Architecture

### 3.1 Frontend

- `src/App.tsx` renders the panel UI
- `src/components/DisplayCard.tsx` renders each display action card
- `src/hooks/useDisplays.ts` loads display state, invokes commands, and listens to `displays-update`

### 3.2 Backend

- `src-tauri/src/commands.rs` orchestrates reads, toggles, wake-all, and frontend events
- `src-tauri/src/overlay.rs` owns overlay window creation and destruction
- `src-tauri/src/panel.rs` handles panel relocation and display ID helpers
- `src-tauri/src/shortcut.rs` registers and unregisters the `Space` shortcut
- `src-tauri/src/state.rs` stores display state, blackout flags, and command coordination flags

---

## 4. State Model

The backend keeps an in-memory map of displays.

Current display state shape:

```text
DisplayState {
  id: String,
  name: String,
  width: u32,
  height: u32,
  x: i32,
  y: i32,
  scale_factor: f64,
  is_primary: bool,
  is_blacked_out: bool,
}
```

Shared application state also tracks:

- whether the global wake shortcut is registered
- whether a toggle command is already in progress
- timing state for double-space detection

The frontend receives a simplified payload with:

- display list
- `activeDisplayCount`
- `blackoutCount`
- per-display flags such as `hostsPanel`, `isBlackedOut`, and `canBlackout`

---

## 5. Overlay System

### 5.1 Overlay Representation

Each blacked-out display is represented by a dedicated native Win32 popup window.

- label format: `overlay-<sanitized-display-id>`
- background: opaque black
- decorations: disabled
- always-on-top: enabled
- activation: disabled with `WS_EX_NOACTIVATE`
- task switching visibility: hidden with `WS_EX_TOOLWINDOW`
- skip taskbar: enabled

### 5.2 Why Overlay Creation Is Asynchronous

Overlay creation is not executed inline inside the toggle command anymore.

Current behavior:

1. `toggle_display` requests overlay creation
2. `show_overlay` spawns a worker thread
3. the worker schedules native window creation on the Tauri main thread
4. the command returns without waiting for the Win32 window call sequence to finish

This avoids the command flow hanging while Windows is creating the new native overlay window.

### 5.3 Overlay Input Behavior

Overlay windows are native popup windows that cover the target display without taking focus from the panel.

---

## 6. Current Command Flows

### 6.1 `get_displays`

Current flow:

1. ensure the cached display list exists
2. if empty, read monitors from Tauri once
3. compute whether the panel currently sits on one of the cached displays using panel window position
4. build payload
5. return payload to the frontend

Important detail: the app no longer refreshes monitor information on every read.

### 6.2 `toggle_display(target_id)`

When blacking out a display:

1. acquire the toggle guard so only one toggle runs at a time
2. read target display from cached state
3. reject if this would black out the last active display
4. if the panel is on the target display, choose a fallback display and move the panel first
5. schedule overlay creation for the target display
6. mark the display as blacked out in backend state
7. sync runtime behaviors such as the wake shortcut
8. emit `displays-update`

When restoring a display:

1. destroy the overlay window for the target display
2. mark the display as active
3. sync runtime behaviors
4. emit `displays-update`

### 6.3 `unblank_all`

Current flow:

1. collect all blacked-out display IDs
2. destroy each overlay window
3. clear blackout flags in backend state
4. sync runtime behaviors
5. emit `displays-update`

---

## 7. Panel Tracking and Relocation

### 7.1 How the Current Panel Display Is Determined

The system does not ask Tauri for the current monitor on every update.

Instead, it:

1. reads the panel window outer position
2. computes the panel center point
3. matches that point against cached display bounds

This avoids extra monitor queries during hot paths.

### 7.2 Fallback Selection

When the panel must move away from the target display, Nocturn picks the nearest active display based on center-point distance.

### 7.3 Current Placement Strategy

The panel is moved to the center of the fallback display using the current fixed panel size constants.

---

## 8. Frontend Behavior

The frontend is intentionally conservative.

- it treats the backend as the source of truth
- it disables actions while a mutation is in progress
- it refreshes state again after commands complete
- it derives the last active display from backend counts and display flags

The `Wake all displays` button is enabled only when at least one display is blacked out and no mutation is in progress.

---

## 9. Known Limitations

- overlay creation is asynchronous, so backend state may update slightly before the blackout window is visibly ready
- display topology is cached after initial load and not fully re-read on every command
- per-display overlay lifecycle is tracked implicitly through window labels rather than a dedicated overlay registry
- logs were temporarily expanded during debugging and can be reduced once the flow is considered stable

---

## 10. Files To Read First

- `src-tauri/src/commands.rs`
- `src-tauri/src/overlay.rs`
- `src-tauri/src/panel.rs`
- `src-tauri/src/shortcut.rs`
- `src/hooks/useDisplays.ts`

---

## 11. Current Invariant

> Nocturn keeps one display active, keeps the panel reachable, and applies native blackout overlays without blocking the panel command flow.
