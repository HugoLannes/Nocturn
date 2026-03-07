# Nocturn - Design Vision

**Version:** v0.1  
**Status:** Working draft  
**Reference mood:** Proton-inspired atmosphere, not Proton mimicry

---

## 1. Why This Document Exists

The PRD defines what the application should do. This document defines how it should feel visually before the build phase starts.

Goals:

- align the overall visual direction of Nocturn
- avoid a result that feels generic or inconsistent
- translate a modern inspiration into reusable design decisions
- prepare a clear handoff for the future frontend implementation

---

## 2. Desired Direction

Nocturn should feel like a tool that is:

- calm
- precise
- premium
- night-first
- lightweight
- system-level

The interface should feel neither like a cold enterprise app nor like a flashy gaming UI. The right balance is a compact, reassuring interface with carefully controlled light.

---

## 3. What We Keep From Proton Inspiration

We mainly keep high-level atmosphere qualities:

- visual depth through layered dark surfaces
- clean, luminous accents
- a subtle hero gradient that adds relief
- clear hierarchy between title, primary CTA, and secondary content
- a polished premium feel without clutter

We do not keep:

- the exact component structure
- the exact layout
- the same colors applied everywhere
- copied microcopy or branding

The right approach is: **borrow the feeling, not the screenshot**.

---

## 4. Translating That Into Nocturn

If Proton suggests a network security product, Nocturn should suggest a night display control tool.

That means Nocturn should be:

- more compact
- more visually quiet
- more utilitarian
- more focused on display state
- darker at its core

The personality of Nocturn can be summarized as:

> "A small, elegant night console for screens."

In practice, this means:

- fewer blocks and less text
- stronger state clarity
- more breathing room around essential actions
- a floating system tray panel feel, not a full mini dashboard

---

## 5. Visual Pillars

### 5.1 Dark Depth

The background should feel deep and almost velvety. Surfaces should not blend into each other.

Intent:

- very dark base background
- slightly elevated surfaces
- soft borders instead of hard separators

### 5.2 Controlled Light

Light should come from accents, the main CTA, and active states.

Intent:

- no constant glow everywhere
- no neon effect across the whole window
- use light as a state or action signal

### 5.3 Fast Readability

The panel should be understandable within seconds.

Intent:

- clearly readable global status at the top
- an obvious main action
- display cards that are easy to scan

### 5.4 Premium Restraint

Every visual effect should exist for a reason.

Intent:

- one strong idea per area
- little to no decorative noise
- a feeling of polish rather than spectacle

---

## 6. Design Keywords

Words to favor:

- deep
- soft
- premium
- quiet
- crisp
- focused
- smooth
- compact

Words to avoid:

- flashy
- futuristic
- aggressive
- overloaded
- cyberpunk
- excessive glassmorphism

---

## 7. Component Vision

### 7.1 Tray Panel

The tray panel is the heart of the experience. It should feel like a floating panel that is dense but breathable.

Principles:

- compact width
- well-rounded corners
- subtle gradient on the upper layer
- calmer body area so the display list can breathe
- discreet footer for secondary actions

### 7.2 Global Status Strip

A small area at the top should summarize the overall situation.

Examples:

- `All displays available`
- `1 display in blackout`
- `Cursor confinement active`

This area should inform without taking too much space.

### 7.3 Primary Action

The primary CTA should be immediately identifiable.

Desired qualities:

- large
- readable
- high contrast
- simple wording

It should not feel like a marketing button. It remains functional first.

### 7.4 Display Cards

Display cards should be the most refined component after the main CTA.

Each card should express:

- display name or index
- resolution
- status
- relative importance, for example `Primary`

Expected behavior:

- fast click action
- crisp feedback
- state that remains visible without animation

### 7.5 Secondary Actions

Secondary actions should exist, but never compete with the main action.

Examples:

- `Open app`
- `Settings` later
- `Quit`

---

## 8. Starter Visual System

### Starter Palette

- background base: `#0a0a0f`
- raised surface: `#12121a`
- soft border: `#242438`
- violet accent: `#7c6aff`
- mint accent: `#39e6c4`
- active green: `#39d98a`
- blackout pink-red: `#ff5d73`
- primary text: `#f5f7fb`
- secondary text: `#a7adcf`

### Typography

- `Outfit` for titles, CTA text, and labels
- `JetBrains Mono` for technical metadata

### Rounding

- panel: `18-20px`
- cards: `14-16px`
- buttons: `12-14px`
- badges: `999px`

### Motion

- short transitions
- soft easing
- no unnecessary looping animations
- glow or elevation only on interaction or meaningful state

---

## 9. UI Tone and Microcopy

The tone should be:

- simple
- direct
- calm
- technical without stiffness

Examples of preferred phrasing:

- `Wake all`
- `Blackout`
- `Primary`
- `Display 1`
- `Ready`

Examples to avoid:

- `Execute display shutdown`
- `Reactivate all connected monitors`
- `Optimal system state detected`

---

## 10. Anti-Patterns to Avoid

- too many gradients across multiple components at once
- too many colors at the same level of importance
- surfaces that are too transparent and hurt readability
- shadows that are too strong
- descriptive text that is too long inside the tray panel
- elements that are too small to scan quickly
- badges and icons without a shared logic

---

## 11. Variation Tracks to Explore Later

- a rose-to-violet hero gradient versus a violet-to-mint one
- very restrained display cards versus slightly illustrated ones
- global status shown as a badge, a sentence, or a pill
- whether to include a simplified multi-display layout miniature
- "minimal panel" mode versus "rich panel" mode

---

## 12. Handoff for the Future Build Phase

### Components to Design

- `TrayPanel`
- `GlobalStatus`
- `PrimaryActionButton`
- `DisplayCard`
- `DisplayStatusBadge`
- `QuickActions`
- `PanelFooter`

### Questions the Future Implementation Should Resolve

- whether to import web fonts or ship with system fallbacks
- the exact acceptable level of glow and shadow
- how to handle long lists when many displays are connected
- keyboard behavior and focus states inside the panel
- how closely the final UI should match future mockups under Tauri constraints

### Visual Quality Checklist

- the global status is understandable at a glance
- the primary action is obvious without overpowering the whole panel
- each display card is readable in both ON and OFF states
- the hierarchy still works even without color
- gradients remain an accent, not a full background texture
- the interface stays clear in a small window
- the UI remains elegant with 1, 2, or 6 displays
- no component relies on animation to be understood

### Design Definition of Done Before Build

- a stable palette exists
- typography roles are defined
- the states of key components are listed
- the base microcopy is consistent
- visual anti-patterns are known
- the direction is clear enough to design and code without starting over
