# Nocturn — Product Requirements Document

**v1.0 — Mars 2026**

| | |
|---|---|
| **Auteur** | Toi + Claude |
| **Plateforme** | Windows (desktop) |
| **Stack** | Tauri + Rust (backend) + React + Tailwind CSS (frontend) |
| **Statut** | Draft |

---

## 1. Problème

La nuit, les écrans secondaires (ou même le principal) émettent une lumière gênante. Il n'existe pas d'app simple et rapide pour « éteindre » visuellement un ou plusieurs écrans en quelques clics, sans réellement couper le signal vidéo (ce qui peut dérégler les positions de fenêtres et la disposition des écrans sous Windows).

## 2. Solution

Une app desktop légère qui vit dans le **system tray** et permet de :

- Voir tous les écrans connectés
- Cliquer sur un écran pour le **blackout** (overlay noir fullscreen)
- **Confiner le curseur** aux écrans actifs pour éviter de bouger la souris sur un écran éteint sans s'en rendre compte
- Tout rallumer d'un coup avec un **double appui sur Espace**

## 3. Utilisateur cible

Toute personne avec un setup multi-écrans qui veut couper la lumière de certains écrans la nuit sans débrancher ni couper le signal.

## 4. Fonctionnalités — MVP

### 4.1 System Tray

- L'app démarre dans le tray (icône à côté de l'horloge)
- **Clic gauche** → ouvre le panneau de contrôle
- **Clic droit** → menu contextuel : Show Panel / Unblank All / Quit
- L'app ne se ferme jamais sauf via Quit — fermer la fenêtre la cache simplement

### 4.2 Panneau de contrôle

- Fenêtre popup positionnée près du tray (coin bas-droit)
- Détection automatique de tous les écrans via `screen.getAllDisplays()`
- Chaque écran affiché comme une **carte cliquable** avec :
  - Résolution (ex: 2560×1440)
  - Badge « Primary » si applicable
  - État visuel clair : **ON** (vert) / **OFF** (rouge)
- Un clic sur une carte = toggle blackout de cet écran
- Bouton « Wake All » pour tout rallumer
- La fenêtre se ferme automatiquement quand on clique ailleurs (blur)

### 4.3 Overlay Blackout

- **WebviewWindow** Tauri en fullscreen sur l'écran ciblé
- Propriétés :
  - `always_on_top: true`
  - Background noir
  - `decorations: false`, `skip_taskbar: true`
  - `focused: false` — ne vole pas le focus
  - Curseur masqué (`cursor: none` en CSS)
- L'overlay ne peut pas être fermé par Alt+F4 (pas de `closable`)

### 4.4 Confinement du curseur

- Quand au moins un écran est blanké, un **polling à ~60fps** vérifie la position du curseur
- Si le curseur entre dans la zone d'un écran blanké → il est repoussé vers l'écran non-blanké le plus proche
- Implémentation via `SetCursorPos` (user32.dll) appelé directement depuis Rust avec la crate `windows-sys` — zéro overhead, pas de process externe
- Le polling s'arrête automatiquement quand tous les écrans sont rallumés

### 4.5 Raccourci global — Double Espace

- **Double appui sur Espace** (intervalle < 350ms) → unblank all
- Le raccourci `Space` est **enregistré uniquement quand au moins un écran est blanké** pour ne pas interférer avec l'utilisation normale du clavier
- Dès que tous les écrans sont rallumés, le raccourci est dés-enregistré

## 5. Architecture technique

```
nocturn/
├── package.json               # Frontend deps (React, Tailwind)
├── src-tauri/                 # Backend Rust (Tauri)
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── src/
│   │   ├── main.rs            # Entry point — setup, tray, global state
│   │   ├── commands.rs        # Tauri commands (get_displays, toggle, etc.)
│   │   ├── overlay.rs         # Création/gestion des fenêtres overlay
│   │   ├── cursor.rs          # Confinement curseur via user32.dll
│   │   └── shortcut.rs        # Double-espace global shortcut
│   └── icons/
├── src/                       # Frontend React
│   ├── App.tsx                # Panneau de contrôle principal
│   ├── components/
│   │   └── DisplayCard.tsx    # Carte écran cliquable
│   ├── hooks/
│   │   └── useDisplays.ts     # Hook Tauri IPC
│   └── index.css              # Tailwind + custom styles
└── public/
    └── overlay.html           # Page noire fullscreen
```

### Stack détaillée

| Composant | Techno | Pourquoi |
|---|---|---|
| Runtime desktop | Tauri 2 | Léger (~5 MB vs ~150 MB Electron), accès natif Windows via Rust |
| Backend | Rust | Contrôle système (curseur, fenêtres), performance du polling curseur |
| UI framework | React | Familiarité, écosystème riche |
| Styling | Tailwind CSS | Rapidité, dark mode natif |
| Cursor control | Rust + windows-sys crate (`SetCursorPos`) | Appel direct user32.dll, zéro overhead (pas de PowerShell) |
| Packaging | Tauri bundler | Build MSI/NSIS installer pour Windows |
| State persistence | Tauri fs + serde | Sauvegarder les préférences (futur) |

### IPC Bridge (Tauri Commands)

```
invoke('get_displays')        → Promise<Display[]>
invoke('toggle_display', id)  → void
invoke('unblank_all')         → void
invoke('close_app')           → void
listen('displays-update', callback) → unlisten
```

Communication via le système de **Tauri Commands** (fonctions Rust annotées `#[tauri::command]`) appelées depuis React avec `@tauri-apps/api`.

## 6. Contraintes et edge cases

| Cas | Comportement attendu |
|---|---|
| Tous les écrans blankés | Le curseur n'est pas confiné (nulle part où aller). Double Espace rallume tout. |
| Écran débranché/rebranché | Le panneau se rafraîchit via `screen.on('display-added/removed')` |
| Alt+Tab avec overlay | L'overlay reste `alwaysOnTop` niveau screen-saver → impossible de passer devant |
| L'app crash | Les overlays sont des fenêtres Tauri → elles disparaissent avec le process, l'écran revient |
| Double Espace en jeu | Le shortcut n'est actif que quand un écran est blanké, donc pas de conflit en usage normal |
| Latence curseur | Polling natif Rust à ~1ms. Quasi imperceptible — bien meilleur qu'un polling JS |

## 7. Design UI

### 7.1 Intentions produit

L'interface doit évoquer un outil :

- **Nocturne** : sombre, reposant, pensé pour un usage de nuit
- **Premium mais léger** : finition propre, sans surcharge visuelle
- **Immédiat** : chaque action importante doit être compréhensible en un regard
- **Discret** : l'app reste en retrait tant qu'on n'en a pas besoin

L'inspiration visuelle recherchée est proche d'une UI moderne type Proton dans l'ambiance generale : surfaces profondes, accents lumineux, gradients subtils, contraste propre et sensation de produit soigne. En revanche, Nocturn doit garder une identite plus utilitaire, plus compacte et plus minimale.

### 7.2 Principes de design

- **Dark mode exclusif** : le produit n'a pas besoin de theme clair
- **Hierarchie forte** : un seul element principal capte l'attention a la fois
- **Densite maitrisee** : interface compacte, mais jamais tassee
- **Lisibilite instantanee** : l'etat de chaque ecran doit se comprendre sans effort
- **Feedback doux** : transitions et effets perceptibles, jamais demonstratifs
- **Coherence stricte** : meme logique visuelle pour les badges, boutons, cartes et quick actions

### 7.3 Fondations visuelles

#### Palette

| Role | Couleur | Usage |
|---|---|---|
| App background | `#0a0a0f` | Fond principal global |
| Elevated surface | `#12121a` | Cartes, panel, sections |
| Surface border | `#242438` | Delimitation douce des blocs |
| Primary accent | `#7c6aff` | CTA principal, focus, accents |
| Secondary accent | `#39e6c4` | Highlights, etat rapide, glow ponctuel |
| Success | `#39d98a` | Ecran actif, confirmation positive |
| Danger | `#ff5d73` | Ecran blackout, action critique |
| Primary text | `#f5f7fb` | Texte principal |
| Secondary text | `#a7adcf` | Texte secondaire, descriptions |

#### Gradients

- Utiliser les gradients avec moderation, surtout sur le hero panel et le CTA principal
- Eviter les backgrounds trop colors sur toute l'interface
- Favoriser des melanges sobres, par exemple `#ff5d73` vers `#7c6aff` pour la couche haute du panneau
- Garder les cartes d'ecran plus neutres pour que les etats ON/OFF restent lisibles

#### Typographie

- **Outfit** pour l'interface principale, titres, labels et CTA
- **JetBrains Mono** pour les resolutions, IDs d'ecrans, infos techniques et metadonnees
- Titres courts, compacts, avec poids visuel assume
- Texte secondaire plus discret mais toujours lisible sur fond sombre

#### Formes, ombres et densite

- Fenetre borderless avec coins arrondis (`16-20px`) et ombre douce
- Cartes et boutons avec rayons coherents (`12-16px`)
- Espacement genereux autour des groupes, compact a l'interieur des composants
- Ombrage surtout utilise pour separer les plans, pas pour decorer

### 7.4 Structure du panneau de controle

Le panneau doit se lire comme une petite console premium en trois niveaux :

1. **Header** : etat global, contexte de l'action principale
2. **Action principale** : bouton clair pour tout rallumer ou action equivalente
3. **Liste des ecrans** : cartes simples, repetables, scannables rapidement

Le panneau ne doit jamais ressembler a une fenetre d'application lourde. Il doit evoquer un objet systeme compact, presque flottant, qui apparait vite, donne confiance, puis disparait.

### 7.5 Etats et interactions

#### Cartes ecran

- **ON** : bordure ou lueur verte, sensation d'ecran disponible, badge clair "On"
- **OFF** : contraste plus faible sur la carte, badge rouge/rose "Blackout", interieur plus sombre
- **Hover** : legere elevation, bordure accentuee, glow discret
- **Pressed** : feedback immediat sans animation molle
- **Focus clavier** : halo violet propre et visible

#### Bouton principal

- Style plus lumineux que le reste de l'UI
- Gradient maitrise et contraste fort
- Largeur confortable pour inspirer confiance
- Microcopy simple, orientee action : `Wake all`, `Restore all` ou equivalent

#### Feedback global

- Les transitions doivent rester courtes (`120-180ms`)
- Les animations servent a confirmer une action, pas a distraire
- Les glows doivent etre subtils et localises
- Aucun effet "gaming" agressif ou neon excessif

### 7.6 Contraintes UX specifiques au produit

- L'utilisateur doit pouvoir identifier en moins de 2 secondes quel ecran est actif ou blackout
- Le panneau doit fonctionner en petite taille sans perdre sa clarte
- Les actions critiques doivent etre accessibles sans navigation supplementaire
- Les labels doivent etre courts et sans jargon inutile
- L'UI doit rester elegante meme avec 1 ecran comme avec 5 ou 6 ecrans

### 7.7 Ton et microcopy

La voix UI doit etre :

- concise
- calme
- technique juste ce qu'il faut
- jamais trop marketing

Exemples de ton :

- `Blackout` plutot que `Disable monitor`
- `Wake all` plutot que `Restore all displays now`
- `Primary` comme badge technique discret
- `Display 1`, `Display 2` ou nom systeme si utile

### 7.8 Garde-fous de coherence

- Ne pas multiplier les couleurs d'etat sans raison
- Ne pas empiler glow + gradient + shadow forte sur un meme composant
- Ne pas faire dependre la comprehension d'une animation
- Ne pas utiliser une iconographie trop decorative
- Conserver un contraste fort entre structure, contenu et actions

## 8. Hors scope (MVP)

- Timer / scheduling (éteindre dans X minutes)
- Profils (sauvegarder des configs d'écrans)
- Brightness dimming progressif
- Support macOS / Linux
- Auto-start avec Windows
- Hotkey personnalisable

## 9. Métriques de succès

C'est un outil perso, donc :

- **Ça marche** : les écrans deviennent noirs en 1 clic
- **Le curseur ne s'échappe pas** sur les écrans éteints
- **Double Espace rallume tout** de manière fiable
- **Pas de bugs d'affichage** au rallumage (fenêtres déplacées, résolutions changées)
- **< 20 MB RAM** au repos dans le tray