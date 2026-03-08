<p align="center">
  <img src="src-tauri/icon-source.svg" alt="Nocturn logo" width="120" />
</p>

<h1 align="center">Nocturn</h1>

<p align="center">
  Black out any screen instantly. Keep the rest of your setup untouched.
</p>

<p align="center">
  <img alt="Version" src="https://img.shields.io/badge/version-0.1.3-111827?style=for-the-badge" />
  <img alt="License" src="https://img.shields.io/badge/license-MIT-111827?style=for-the-badge" />
</p>

<p align="center">
  <img src="src/assets/preview-1.png" alt="Nocturn – display blackout panel" width="960" />
</p>

---

Nocturn is a lightweight Windows tray utility for multi-monitor setups. It places a native fullscreen black overlay on any display you choose — without cutting the video signal, moving your windows, or changing your desktop topology.

Useful at night, during focused work, or anytime a spare monitor is just extra light. On OLED panels, the pure black blackout is OLED-friendly and keeps black pixels effectively off.

## Features

- **Per-display blackout** — darken one screen or several at once with a single click.
- **Native overlay engine** — uses native Windows overlays for instant blackout behavior.
- **OLED-friendly blackout** — pure black overlays are ideal for OLED screens.
- **Instant restore** — bring everything back immediately, or wake all displays at once.
- **Safety guard** — always keeps at least one display active; the panel moves out of the way automatically.
- **Tray-resident** — runs quietly in the system tray, opens on click.
- **Auto-updater** — checks for new releases and updates in the background.

## Install

Download the latest `.msi` installer from [**GitHub Releases**](https://github.com/HugoLannes/nocturn/releases/latest).

## Build from source

Nocturn is built with [Tauri 2](https://tauri.app/), React, and Rust.

```bash
# prerequisites: Node >=18, Rust, Tauri CLI
npm install
npm run tauri:dev      # development
npm run tauri:build    # production installer
```

## Tech stack

| Layer    | Technology              |
|----------|-------------------------|
| Shell    | Tauri 2 (Rust)          |
| Frontend | React 19 · TypeScript · Tailwind CSS 4 |
| Build    | Vite 7                  |

## Contributing

Issues and pull requests are welcome. If you run into a bug or have a feature idea, [open an issue](https://github.com/HugoLannes/nocturn/issues).

## License

[MIT](LICENSE)
