# Frontend Redesign — Nocturn UI

**Date:** 2026-03-07
**Status:** Approved

## Direction

Dark glass aesthetic. Fond très sombre, surfaces semi-transparentes, accents indigo avec états vert/rouge pour les displays. Fenêtre sans decorations natives, titlebar custom intégrée.

## Palette

| Rôle | Valeur |
|------|--------|
| Fond fenêtre | `#0d0d12` |
| Surface card | `rgba(255,255,255,0.04)` |
| Border card ON | `rgba(52,211,153,0.3)` |
| Border card OFF | `rgba(239,68,68,0.3)` |
| Dot ON | `#34d399` |
| Dot OFF | `#ef4444` |
| Texte primaire | `#f1f5f9` |
| Texte secondaire | `#64748b` |
| Accent | `#6366f1` |
| Titlebar border | `rgba(255,255,255,0.06)` |

## Layout

Fenêtre 420×640. 3 zones :
1. **Titlebar** (44px, drag region) — logo + nom + statut à gauche, boutons `−`/`×` à droite
2. **Scrollable content** — liste de display cards
3. **Footer fixe** — bouton "Wake all" pleine largeur

## Composants

### Titlebar
- Toute la largeur est drag region sauf les boutons
- Gauche : icône ⚡ + "Nocturn" semibold + dot coloré + label "Ready" ou "X in blackout"
- Droite : bouton minimize (−) et close (×), 28×28px, rounded
- Séparateur border-bottom subtil

### Display cards
- Nettoyage des noms Windows : `\\.\DISPLAY4` → `Display 4`
- Padding 14px, border-radius 12px, border 1px
- Gauche : nom bold + résolution monospace + badge "Primary" si applicable
- Droite : dot coloré + label "ON" / "OFF"
- Hover : border plus lumineuse
- Disabled : opacity 50%, cursor not-allowed

### Wake all button
- Footer fixe, pleine largeur, hauteur 48px
- Background indigo `#6366f1`
- Disabled si `blackoutCount === 0`

## Fichiers à modifier

- `src/App.tsx` — restructurer layout, titlebar, invoke commands
- `src/index.css` — réécrire styles
- `src/components/DisplayCard.tsx` — nouveau design card
