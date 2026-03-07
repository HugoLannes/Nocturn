# Nocturn - Display Safety Technical Spec

**Version:** v0.1  
**Status:** Draft  
**Scope:** MVP behavior for panel placement and blackout safety rules

---

## 1. Problem Statement

Nocturn must prevent the control panel from becoming inaccessible when a display is blacked out.

Two product constraints must be enforced together:

1. The app window must never remain on a display that is being blacked out.
2. The user must never be allowed to black out all displays. At least one display must remain active at all times.

This means the product should follow an **`n-1` rule**:

- if `n` displays are connected, at most `n-1` displays can be blacked out
- one display must always remain active
- if the panel currently sits on the display being turned off, it must move to another active display before blackout is finalized

---

## 2. Goals

- keep the control panel reachable at all times
- avoid trapping the UI behind a blackout overlay
- simplify cursor confinement logic by guaranteeing at least one valid destination
- preserve a predictable UX when toggling displays quickly

## 3. Non-Goals

- supporting full blackout of every connected display in the MVP
- solving advanced multi-window scenarios beyond the main tray panel
- persisting per-display panel placement preferences

---

## 4. Product Rules

### 4.1 Core Rule: `n-1` Maximum Blackout

The system must reject any action that would result in all connected displays becoming blacked out.

Examples:

- `1` connected display -> `0` displays may be blacked out
- `2` connected displays -> at most `1` display may be blacked out
- `3` connected displays -> at most `2` displays may be blacked out

### 4.2 Panel Reachability Rule

Before a blackout is applied to a display, Nocturn must verify whether the main control panel is currently visible on that display.

If yes:

- select another active display
- move the panel to that display
- only then create or reveal the blackout overlay on the original display

### 4.3 Last Active Display Rule

If the targeted display is the last currently active display, the blackout action must not run.

Expected UX:

- the toggle is prevented
- the UI stays responsive
- the user receives a short explanation such as `At least one display must stay active`

---

## 5. Proposed UX Behavior

### 5.1 When Toggling a Non-Panel Display Off

If the user blacks out a display that does not contain the panel:

- validate the `n-1` rule
- create the overlay
- update application state
- keep the panel where it is

### 5.2 When Toggling the Panel Display Off

If the user blacks out the display that currently contains the visible panel:

- validate that another active display exists
- pick a fallback display
- move the panel to the fallback display
- wait until the panel move is complete
- apply the blackout overlay
- update state

The move should feel immediate and intentional, not like a flicker.

### 5.3 When the User Tries to Black Out the Last Active Display

If the action would leave zero active displays:

- do not move the panel
- do not create the overlay
- keep the display active
- show a lightweight error or disabled-state message

---

## 6. Technical Approach

### 6.1 High-Level Flow

```text
User toggles display off
-> load current display state
-> compute active display count
-> if action violates n-1 rule: reject
-> if panel is on target display: relocate panel
-> create blackout overlay
-> update shared state
-> refresh frontend
```

### 6.2 Suggested State Model

Backend state should track at least:

- connected displays
- blackout state per display
- active overlays per display
- current panel display ID when panel is visible
- optional last-known panel bounds

Example shape:

```text
DisplayState {
  id: String,
  bounds: Rect,
  is_primary: bool,
  is_blacked_out: bool,
}

AppState {
  displays: Vec<DisplayState>,
  panel_display_id: Option<String>,
  panel_visible: bool,
}
```

### 6.3 Recommended Command Sequence

For `toggle_display(target_id)`:

1. Read current display state.
2. If target is currently active, compute whether blacking it out would violate the `n-1` rule.
3. If invalid, return an explicit domain error such as `CannotBlackoutLastActiveDisplay`.
4. If the panel is visible and currently hosted on `target_id`, relocate it first.
5. Create the overlay window for `target_id`.
6. Mark the display as blacked out.
7. Emit a frontend update event.

For `toggle_display(target_id)` when restoring:

1. Destroy or hide the overlay for `target_id`.
2. Mark the display as active.
3. Emit a frontend update event.

---

## 7. Panel Relocation Strategy

### 7.1 Fallback Display Selection

When the panel must move, choose a fallback display using a deterministic order:

1. another non-blacked-out display nearest to the current panel display
2. otherwise the primary active display
3. otherwise the first active display in the current display list

This keeps behavior predictable and reduces surprising jumps.

### 7.2 Relocation Timing

The relocation must happen **before** blackout is made visible on the target display.

Preferred sequence:

1. compute fallback display
2. reposition panel window inside fallback bounds
3. confirm panel window target update
4. apply blackout overlay to the original display

### 7.3 Placement on the Fallback Display

The panel should preserve its anchoring behavior relative to the tray as much as possible, but MVP safety is more important than pixel-perfect continuity.

MVP fallback behavior:

- place the panel near the bottom-right corner of the fallback display
- clamp the window inside visible bounds
- avoid placing it under the taskbar if that can be detected

---

## 8. Failure Handling

### 8.1 No Fallback Display Available

This should only happen if the `n-1` rule is broken or the display state is stale.

Fallback behavior:

- abort the blackout action
- keep the target display active
- log the failure
- surface a safe user-facing message if possible

### 8.2 Relocation Fails

If panel relocation fails for any reason:

- abort the blackout action
- do not create the overlay
- keep UI access on the current display

This is safer than blacking out the display and risking an inaccessible panel.

### 8.3 Display Topology Changes Mid-Action

If a display is unplugged or the topology changes during relocation:

- re-read the display list
- recompute active displays
- if a valid fallback still exists, retry once
- otherwise abort the action safely

---

## 9. Edge Cases

| Case | Expected behavior |
|---|---|
| Single-display setup | Blackout action is unavailable because it would violate the `n-1` rule. |
| Two-display setup, panel on display A, user blacks out A | Panel moves to display B, then display A is blacked out. |
| Two-display setup, display B already blacked out, user tries to black out A | Action is rejected because A is the last active display. |
| Panel hidden in tray | No relocation is needed; only the `n-1` rule applies. |
| Rapid repeated toggles | Commands should serialize or ignore conflicting requests while relocation / overlay creation is in progress. |
| Display unplugged after relocation target is chosen | Recompute fallback or abort safely. |

---

## 10. Suggested Backend Responsibilities

- `commands.rs`
  - validate toggle requests
  - return explicit domain errors
  - orchestrate relocation + overlay sequencing

- `overlay.rs`
  - create and destroy blackout overlays
  - expose whether a display currently has an overlay

- `main.rs`
  - own shared application state
  - track panel window visibility and current display

- possible future `panel.rs`
  - encapsulate panel positioning, relocation, and clamping logic

---

## 11. Suggested Frontend Responsibilities

- disable or visually guard the last remaining active display from being turned off
- present a short explanatory message when a blackout is rejected
- refresh card states immediately after backend events
- avoid optimistic UI that assumes blackout succeeded before backend confirmation

---

## 12. Open Decisions

- whether the last active display should appear fully disabled or still clickable with an explanatory tooltip
- whether relocation should animate or remain instant in MVP
- whether panel display tracking should rely on Tauri window position only or also maintain explicit backend state
- whether toggle requests should be mutex-protected during relocation

---

## 13. Recommended MVP Decision

For MVP, the safest and simplest implementation is:

- enforce the `n-1` rule in the backend as the source of truth
- also reflect the rule in the frontend by disabling the invalid toggle
- relocate the panel instantly, without animation
- abort blackout if relocation cannot be completed

This gives the product a clear invariant:

> Nocturn always keeps one active display, and the control panel never disappears behind a blackout.
