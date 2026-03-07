# Nocturn

![Status](https://img.shields.io/badge/status-prototype-3b82f6?style=for-the-badge)
![Platform](https://img.shields.io/badge/platform-Windows-0078D6?style=for-the-badge&logo=windows)
![Tauri](https://img.shields.io/badge/Tauri-2.x-FFC131?style=for-the-badge&logo=tauri&logoColor=000000)
![Rust](https://img.shields.io/badge/Rust-backend-000000?style=for-the-badge&logo=rust)
![React](https://img.shields.io/badge/React-frontend-61DAFB?style=for-the-badge&logo=react&logoColor=000000)

Nocturn is a Windows tray app that blacks out one or more displays with fullscreen black overlays without disabling the video signal.

## What It Does Today

- lists connected displays in a compact control panel
- toggles a per-display blackout overlay on and off
- keeps at least one display active at all times
- moves the panel away before blacking out the display that currently hosts it
- wakes every blacked-out display from the panel or with a double-tap on `Space`

## Current Architecture

- `src/` contains the React panel UI and the display state hook
- `src-tauri/src/commands.rs` is the main orchestration layer for display actions
- `src-tauri/src/overlay.rs` creates per-display native blackout windows
- `src-tauri/src/panel.rs` handles panel positioning and relocation
- `src-tauri/src/shortcut.rs` registers the wake shortcut when at least one display is blacked out

## Important Implementation Notes

- overlays are separate native Win32 popup windows created from Rust
- overlay creation is scheduled asynchronously onto Tauri's main thread so the panel command flow stays responsive
- the backend is the source of truth for blackout state and safety rules
- the frontend waits for backend confirmation and refreshes from `displays-update` events
- cursor confinement is currently disabled

## Stack

- `Tauri 2`
- `Rust`
- `React`
- `TypeScript`
- `Vite`

## Documentation

- `docs/product-requirements.md`
- `docs/design-vision.md`
- `docs/display-safety-technical-spec.md`

## Dev Notes

- restart `tauri dev` after Rust changes in `src-tauri/`
- use `cargo check` in `src-tauri/` to validate backend changes quickly
