# Nocturn - Product Requirements

**Version:** v0.2  
**Platform:** Windows desktop  
**Stack:** Tauri + Rust + React + TypeScript  
**Status:** Prototype aligned with current implementation

---

## 1. Problem

At night, some displays in a multi-monitor setup can become unwanted light sources. Turning monitors off physically or disabling them in Windows often breaks window placement, desktop topology, or workflow continuity.

Nocturn solves the narrower problem: make a display appear off without cutting the video signal.

---

## 2. Product Goal

Provide a fast tray utility that lets the user black out one or more displays, keep one display usable, and restore everything instantly.

---

## 3. Core User Value

The user can:

- see all connected displays in one compact panel
- black out a display with one click
- restore a display with one click
- restore every blacked-out display at once
- keep the control panel reachable while using multiple monitors

---

## 4. Current Functional Requirements

### 4.1 Tray App

- The app runs as a Windows tray utility.
- Left click on the tray icon opens the panel.
- Tray menu exposes `Show Panel`, `Wake All`, and `Quit`.

### 4.2 Display Panel

- The panel lists detected displays.
- Each display appears as a clickable card with name, resolution, and state.
- The panel exposes a `Wake all displays` action.
- The panel must remain usable while blackout actions run.

### 4.3 Blackout Behavior

- Blackout is implemented with a fullscreen black overlay window per display.
- Overlays must visually cover the target display.
- Overlays must not disable the actual monitor signal.
- Restoring a display removes its overlay.

### 4.4 Safety Rules

- At least one display must remain active at all times.
- If the panel is currently on the display being blacked out, the panel must move first.
- Toggle actions must serialize so conflicting display operations do not run in parallel.

### 4.5 Wake Behavior

- `Wake all displays` restores every blacked-out display.
- Double-pressing `Space` also restores every blacked-out display.
- The global shortcut is only registered while at least one display is blacked out.

---

## 5. Non-Requirements For The Current Version

The current prototype does not require:

- cursor confinement
- scheduled blackout timers
- per-display profiles
- persistent panel placement preferences
- cross-platform support
- graceful handling of every live topology change during runtime

---

## 6. Current UX Expectations

- The user should understand which displays are active and blacked out immediately.
- Clicking a display should feel direct and responsive.
- Rejecting the last active display blackout should be clear and safe.
- Restoring all displays should be a single obvious action.

---

## 7. Technical Requirements

- The backend is the source of truth for display safety rules.
- The frontend must not assume success before backend confirmation.
- Overlay creation must not block the panel command flow.
- Display state changes must be pushed back to the frontend through Tauri events.

---

## 8. Current Architecture Requirements

- `src-tauri/src/commands.rs` orchestrates display actions and emits state updates.
- `src-tauri/src/overlay.rs` creates and destroys blackout overlays.
- `src-tauri/src/panel.rs` moves the panel between displays when needed.
- `src-tauri/src/shortcut.rs` manages the temporary wake shortcut.
- `src/hooks/useDisplays.ts` drives frontend loading, toggling, and wake-all flows.

---

## 9. Current Product Constraints

- Windows only
- one main panel window
- blackout overlays are separate Tauri webview windows
- display topology is cached after initial load rather than fully re-read on every command

---

## 10. Success Criteria

The current version is successful if:

- a target display becomes black reliably
- the panel stays reachable
- the app does not hang when an overlay is created
- `Wake all displays` restores all blacked-out displays reliably
- the double-space shortcut works only when needed
