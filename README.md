# Nocturn

![Status](https://img.shields.io/badge/status-draft-7c6aff?style=for-the-badge)
![Platform](https://img.shields.io/badge/platform-Windows-0078D6?style=for-the-badge&logo=windows)
![Tauri](https://img.shields.io/badge/Tauri-2.x-FFC131?style=for-the-badge&logo=tauri&logoColor=000000)
![Rust](https://img.shields.io/badge/Rust-backend-000000?style=for-the-badge&logo=rust)
![React](https://img.shields.io/badge/React-frontend-61DAFB?style=for-the-badge&logo=react&logoColor=000000)
![Tailwind CSS](https://img.shields.io/badge/Tailwind_CSS-styling-06B6D4?style=for-the-badge&logo=tailwindcss&logoColor=ffffff)

Nocturn est une application desktop Windows pensee pour couper visuellement un ou plusieurs ecrans en un clic, sans desactiver le signal video.

L'objectif est simple : reduire la lumiere parasite la nuit, garder un setup multi-ecrans propre, et retrouver tous les ecrans instantanement quand il faut.

## Concept

Nocturn vit dans le `system tray` et permet de :

- voir les ecrans connectes
- activer un blackout plein ecran sur un display
- confiner le curseur aux ecrans actifs
- tout rallumer rapidement avec un double appui sur `Espace`

## Stack

- `Tauri` pour le runtime desktop
- `Rust` pour la logique systeme et le controle natif Windows
- `React` pour l'interface
- `Tailwind CSS` pour le styling

## Direction produit

- interface sombre, moderne et elegante
- gradients subtils et surfaces profondes
- panel compact type tray utility
- hierarchie visuelle claire et micro-interactions discretes

## Etat actuel

Le projet est pour l'instant en phase de documentation et de design.

- PRD en cours
- vision design posee
- build a venir

## Documentation

- `docs/# Nocturn - Product Requirements Documen.md`
- `docs/Nocturn - Design Vision.md`

## Roadmap immediate

- finaliser la direction UX/UI
- cadrer les composants du tray panel
- lancer ensuite le build de la premiere version MVP
