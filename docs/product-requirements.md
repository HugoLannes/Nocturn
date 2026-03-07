# Nocturn - Product Requirements Document

**v1.0 - March 2026**

| | |
|---|---|
| **Author** | You + Claude |
| **Platform** | Windows (desktop) |
| **Stack** | Tauri + Rust (backend) + React + Tailwind CSS (frontend) |
| **Status** | Draft |

---

## 1. Problem

At night, secondary displays, or even the main one, can emit distracting light. There is no simple, fast app to visually "turn off" one or more screens in a few clicks without actually cutting the video signal, which can mess up window placement and display layout on Windows.

## 2. Solution

A lightweight desktop app that lives in the **system tray** and lets you:

- See all connected displays
- Click a display to trigger a **blackout** with a fullscreen black overlay
- **Confine the cursor** to active screens so you do not accidentally move the mouse onto a blacked-out display
- Wake everything back up with a **double tap on Space**

## 3. Target User

Anyone with a multi-display setup who wants to reduce light from some screens at night without unplugging them or cutting the signal.

## 4. Features - MVP

### 4.1 System Tray

- The app starts in the tray
- **Left click** opens the control panel
- **Right click** opens a context menu: Show Panel / Unblank All / Quit
- The app only exits through Quit; closing the window simply hides it

### 4.2 Control Panel

- Popup window positioned near the tray in the bottom-right corner
- Automatic detection of all displays via `screen.getAllDisplays()`
- Each display is shown as a **clickable card** with:
  - Resolution, for example `2560x1440`
  - A `Primary` badge when applicable
  - A clear visual state: **ON** (green) / **OFF** (red)
- Clicking a card toggles blackout for that display
- A `Wake All` button turns everything back on
- The window automatically closes when the user clicks elsewhere

### 4.3 Blackout Overlay

- A Tauri **WebviewWindow** in fullscreen on the targeted display
- Properties:
  - `always_on_top: true`
  - Black background
  - `decorations: false`, `skip_taskbar: true`
  - `focused: false` so it does not steal focus
  - Hidden cursor with `cursor: none` in CSS
- The overlay cannot be closed with Alt+F4 and should not be user-closable

### 4.4 Cursor Confinement

- When at least one display is blacked out, a **~60fps polling loop** checks the cursor position
- If the cursor enters a blacked-out display, it gets pushed back to the nearest non-blacked-out display
- Implemented with `SetCursorPos` from `user32.dll` called directly from Rust via `windows-sys`, with no external process
- Polling stops automatically when all displays are active again

### 4.5 Global Shortcut - Double Space

- **Double tap on Space** with an interval below `350ms` triggers `unblank all`
- The `Space` shortcut is **registered only while at least one display is blacked out** so normal keyboard usage is unaffected
- As soon as all displays are restored, the shortcut is unregistered

## 5. Technical Architecture

```text
nocturn/
├── package.json               # Frontend dependencies (React, Tailwind)
├── src-tauri/                 # Rust backend (Tauri)
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── src/
│   │   ├── main.rs            # Entry point - setup, tray, global state
│   │   ├── commands.rs        # Tauri commands (get_displays, toggle, etc.)
│   │   ├── overlay.rs         # Overlay window creation and management
│   │   ├── cursor.rs          # Cursor confinement through user32.dll
│   │   └── shortcut.rs        # Global double-space shortcut
│   └── icons/
├── src/                       # React frontend
│   ├── App.tsx                # Main control panel
│   ├── components/
│   │   └── DisplayCard.tsx    # Clickable display card
│   ├── hooks/
│   │   └── useDisplays.ts     # Tauri IPC hook
│   └── index.css              # Tailwind + custom styles
└── public/
    └── overlay.html           # Fullscreen black page
```

### Detailed Stack

| Component | Technology | Why |
|---|---|---|
| Desktop runtime | Tauri 2 | Lightweight compared with Electron, with native Windows access through Rust |
| Backend | Rust | System control, window management, cursor polling performance |
| UI framework | React | Familiar ecosystem and fast UI iteration |
| Styling | Tailwind CSS | Fast styling workflow and dark-mode-friendly |
| Cursor control | Rust + `windows-sys` (`SetCursorPos`) | Direct `user32.dll` calls with no PowerShell overhead |
| Packaging | Tauri bundler | Windows installer output via MSI or NSIS |
| State persistence | Tauri fs + serde | Save preferences in future versions |

### IPC Bridge (Tauri Commands)

```text
invoke('get_displays')        -> Promise<Display[]>
invoke('toggle_display', id)  -> void
invoke('unblank_all')         -> void
invoke('close_app')           -> void
listen('displays-update', callback) -> unlisten
```

Communication happens through **Tauri Commands**, which are Rust functions annotated with `#[tauri::command]` and called from React using `@tauri-apps/api`.

## 6. Constraints and Edge Cases

| Case | Expected behavior |
|---|---|
| All displays blacked out | The cursor is not confined because there is nowhere valid to send it. Double Space restores everything. |
| Display unplugged / replugged | The panel refreshes through `screen.on('display-added/removed')`. |
| Alt+Tab while an overlay exists | The overlay stays above everything because it behaves like a screen-saver-level always-on-top window. |
| App crash | Overlays disappear with the process, so the screen becomes visible again. |
| Double Space during gaming | The shortcut is active only while at least one display is blacked out, reducing conflicts in normal use. |
| Cursor latency | Native Rust polling around `~1ms`; effectively imperceptible and better than a JS loop. |

## 7. UI Design

### 7.1 Product Intent

The interface should feel like a tool that is:

- **Night-first**: dark, restful, designed for late use
- **Premium but light**: polished without visual overload
- **Immediate**: each important action should be understood at a glance
- **Discrete**: the app stays out of the way until needed

The visual inspiration should stay close to a modern Proton-like atmosphere: deep surfaces, luminous accents, subtle gradients, clean contrast, and a carefully crafted product feel. At the same time, Nocturn must stay more utilitarian, more compact, and more minimal.

### 7.2 Design Principles

- **Dark mode only**: the product does not need a light theme
- **Strong hierarchy**: only one main element should attract attention at a time
- **Controlled density**: compact, but never cramped
- **Instant readability**: the state of each display must be obvious
- **Soft feedback**: transitions and effects should be noticeable but restrained
- **Strict consistency**: badges, buttons, cards, and quick actions should follow the same visual logic

### 7.3 Visual Foundations

#### Palette

| Role | Color | Usage |
|---|---|---|
| App background | `#0a0a0f` | Main application background |
| Elevated surface | `#12121a` | Cards, panel, sections |
| Surface border | `#242438` | Soft separation between blocks |
| Primary accent | `#7c6aff` | Primary CTA, focus, accents |
| Secondary accent | `#39e6c4` | Highlights, quick status cues, occasional glow |
| Success | `#39d98a` | Active display, positive confirmation |
| Danger | `#ff5d73` | Blacked-out display, critical action |
| Primary text | `#f5f7fb` | Main text |
| Secondary text | `#a7adcf` | Supporting text and descriptions |

#### Gradients

- Use gradients sparingly, mainly on the hero area of the panel and the primary CTA
- Avoid heavily colored backgrounds across the whole interface
- Favor restrained blends, for example `#ff5d73` to `#7c6aff` on the upper panel layer
- Keep display cards more neutral so ON/OFF states remain easy to read

#### Typography

- **Outfit** for the main interface, titles, labels, and CTA text
- **JetBrains Mono** for resolutions, display IDs, and technical metadata
- Short, compact headings with clear visual weight
- Secondary text should stay discreet but readable on dark surfaces

#### Shapes, Shadows, and Density

- Borderless window with rounded corners (`16-20px`) and a soft shadow
- Consistent radii on cards and buttons (`12-16px`)
- Generous spacing around groups, tighter spacing inside components
- Shadows should separate layers, not act as decoration

### 7.4 Control Panel Structure

The panel should read like a small premium console with three levels:

1. **Header**: global status and context for the main action
2. **Primary action**: a clear button to wake everything or perform the main action
3. **Display list**: simple, repeatable cards that can be scanned quickly

The panel should never feel like a heavy desktop window. It should feel like a compact system object that appears quickly, inspires confidence, and disappears again.

### 7.5 States and Interactions

#### Display Cards

- **ON**: green border or glow, feeling of availability, clear `On` badge
- **OFF**: lower contrast, pink-red `Blackout` badge, darker interior
- **Hover**: slight lift, clearer border, subtle glow
- **Pressed**: immediate feedback, no sluggish animation
- **Keyboard focus**: clean, visible violet halo

#### Primary Button

- Brighter than the rest of the UI
- Controlled gradient and strong contrast
- Comfortable width that inspires confidence
- Simple, action-oriented microcopy such as `Wake all`, `Restore all`, or similar

#### Global Feedback

- Transitions should stay short (`120-180ms`)
- Animations confirm actions instead of distracting from them
- Glows should be subtle and localized
- No aggressive gaming-like or excessive neon effects

### 7.6 Product-Specific UX Constraints

- The user should identify active vs blacked-out displays in under 2 seconds
- The panel must remain clear even at a small size
- Critical actions should stay accessible without extra navigation
- Labels should stay short and free from unnecessary jargon
- The UI must remain elegant with 1 display as well as with 5 or 6 displays

### 7.7 Voice and Microcopy

The UI voice should be:

- concise
- calm
- technical only where useful
- never overly marketing-driven

Examples:

- `Blackout` instead of `Disable monitor`
- `Wake all` instead of `Restore all displays now`
- `Primary` as a discreet technical badge
- `Display 1`, `Display 2`, or the system name when useful

### 7.8 Consistency Guardrails

- Do not multiply state colors without a reason
- Do not stack glow + gradient + strong shadow on the same component
- Do not rely on animation for comprehension
- Do not use overly decorative iconography
- Maintain strong contrast between structure, content, and actions

## 8. Out of Scope (MVP)

- Timer / scheduling, for example black out in X minutes
- Profiles to save display configurations
- Progressive brightness dimming
- macOS / Linux support
- Windows auto-start
- Customizable hotkey

## 9. Success Metrics

This is a personal tool, so success means:

- **It works**: displays turn black in 1 click
- **The cursor does not escape** onto blacked-out screens
- **Double Space restores everything** reliably
- **No visual glitches** when waking displays back up, such as moved windows or changed resolutions
- **Below 20 MB RAM** while idle in the tray
