# Nocturn - Design Vision

**Version:** v0.1  
**Statut:** Working draft  
**Reference mood:** Proton-inspired atmosphere, not Proton mimicry

---

## 1. Pourquoi ce document

Le PRD fixe ce que l'application doit faire. Ce document sert a cadrer comment elle doit se ressentir visuellement avant la phase de build.

Objectifs :

- aligner l'ambiance generale de Nocturn
- eviter un rendu trop generique ou incoherent
- traduire une inspiration moderne en decisions reutilisables
- preparer un handoff clair pour la future implementation frontend

---

## 2. Direction souhaitee

Nocturn doit donner l'impression d'un outil :

- calme
- precis
- premium
- nocturne
- leger
- system-level

L'interface ne doit ni ressembler a une app enterprise froide, ni a une UI gaming tape-a-l'oeil. Le bon point d'equilibre est une interface compacte, rassurante, avec une touche lumineuse maitrisee.

---

## 3. Ce qu'on retient de l'inspiration Proton

On retient surtout des qualites d'ambiance :

- profondeur visuelle avec surfaces sombres superposees
- accents colores propres et lumineux
- gradient hero discret pour donner du relief
- hierarchie lisible entre titre, CTA principal et contenu secondaire
- impression de finition haut de gamme sans surcharge

On ne retient pas :

- la structure exacte des composants
- la disposition exacte des blocs
- les memes couleurs appliquees partout
- une copie de microcopy ou de branding

La bonne approche est : **s'inspirer de la sensation, pas du screenshot**.

---

## 4. Traduction pour Nocturn

Si Proton evoque un produit de securite reseau, Nocturn doit evoquer un outil de controle d'ecrans de nuit.

Donc Nocturn doit etre :

- plus compact
- plus silencieux visuellement
- plus utilitaire
- plus centre sur l'etat des displays
- plus sombre dans sa base

La personnalite de Nocturn peut se resumer ainsi :

> "A small, elegant night console for screens."

En pratique, cela implique :

- moins de blocs et de texte
- plus de clarte d'etat
- plus de respiration autour des actions essentielles
- une sensation de panneau flottant de system tray, pas de mini dashboard complet

---

## 5. Piliers visuels

### 5.1 Dark depth

Le fond doit etre profond et presque veloute. Les surfaces ne doivent pas se confondre entre elles.

Intention :

- fond tres sombre
- surfaces legerement relevees
- bordures douces plutot que separations dures

### 5.2 Controlled light

La lumiere doit venir des accents, du CTA principal et des etats actifs.

Intention :

- pas de glow permanent partout
- pas d'effet neon sur toute la fenetre
- utiliser la lumiere comme signal de statut ou d'action

### 5.3 Fast readability

Le panneau doit pouvoir etre compris en quelques secondes.

Intention :

- statut global lisible en haut
- action principale tres evidente
- cartes displays faciles a scanner

### 5.4 Premium restraint

Chaque effet visuel doit avoir une raison.

Intention :

- une seule idee forte par zone
- peu de decoration gratuite
- sensation de finition plutot que de demonstration

---

## 6. Mots-cles de design

Mots a favoriser :

- deep
- soft
- premium
- quiet
- crisp
- focused
- smooth
- compact

Mots a eviter :

- flashy
- futuristic
- aggressive
- overloaded
- cyberpunk
- glassmorphism excessif

---

## 7. Components vision

### 7.1 Tray panel

Le tray panel est le coeur de l'experience. Il doit ressembler a un panneau flottant, dense mais aere.

Principes :

- largeur compacte
- coins bien arrondis
- couche haute avec gradient discret
- corps plus sobre pour laisser respirer la liste d'ecrans
- footer discret pour les actions secondaires

### 7.2 Global status strip

Une petite zone en haut doit resumer la situation generale.

Exemples de role :

- `All displays available`
- `1 display in blackout`
- `Cursor confinement active`

Cette zone doit informer sans prendre trop de place.

### 7.3 Primary action

Le CTA principal doit etre immediatement identifiable.

Qualites souhaitees :

- grand
- lisible
- contrastant
- simple en wording

Le CTA ne doit pas avoir l'air d'un bouton marketing. Il reste fonctionnel avant tout.

### 7.4 Display cards

Les cartes doivent etre le composant le plus soigne apres le CTA.

Chaque carte doit exprimer :

- nom ou index du display
- resolution
- statut
- importance relative, par exemple `Primary`

Comportement attendu :

- clic rapide
- feedback net
- etat visible meme sans animation

### 7.5 Secondary actions

Les actions secondaires doivent etre presentes mais jamais concurrentes avec l'action principale.

Exemples :

- `Open app`
- `Settings` plus tard
- `Quit`

---

## 8. Visual system de depart

### Palette de depart

- background base : `#0a0a0f`
- raised surface : `#12121a`
- border soft : `#242438`
- accent violet : `#7c6aff`
- accent mint : `#39e6c4`
- active green : `#39d98a`
- blackout pink-red : `#ff5d73`
- text primary : `#f5f7fb`
- text secondary : `#a7adcf`

### Typographies

- `Outfit` pour les titres, CTA et labels
- `JetBrains Mono` pour les metadonnees techniques

### Rounding

- panel : `18-20px`
- cards : `14-16px`
- buttons : `12-14px`
- badges : `999px`

### Motion

- transitions courtes
- easing douce
- pas d'animations en boucle inutiles
- glow ou elevation seulement sur interaction ou etat

---

## 9. Ton UI et microcopy

Le ton doit etre :

- simple
- direct
- calme
- technique sans rigidite

Exemples de formulations souhaitables :

- `Wake all`
- `Blackout`
- `Primary`
- `Display 1`
- `Ready`

Exemples a eviter :

- `Execute display shutdown`
- `Reactivate all connected monitors`
- `Optimal system state detected`

---

## 10. Anti-patterns a eviter

- trop de gradients sur plusieurs composants a la fois
- trop de couleurs au meme niveau d'importance
- surfaces trop transparentes qui reduisent la lisibilite
- ombres trop fortes
- texte descriptif trop long dans le tray panel
- elements trop petits pour etre scannes rapidement
- badges et icones sans logique commune

---

## 11. Pistes de variations a brainstormer plus tard

- gradient hero plutot rose-violet ou violet-mint
- cartes displays tres sobres ou legerement illustrees
- affichage du statut global sous forme de badge, phrase ou pill
- presence ou non d'une miniature simplifiee de layout multi-ecrans
- mode "minimal panel" versus mode "rich panel"

---

## 12. Handoff pour la future phase de build

### Composants a concevoir

- `TrayPanel`
- `GlobalStatus`
- `PrimaryActionButton`
- `DisplayCard`
- `DisplayStatusBadge`
- `QuickActions`
- `PanelFooter`

### Questions que la future implementation devra trancher

- importer des polices web ou embarquer des fallbacks systeme
- niveau exact de glow et d'ombre acceptable
- gestion de listes longues si beaucoup d'ecrans
- comportement clavier et focus states dans le panneau
- niveau de fidelity entre maquette et contraintes Tauri

### Checklist qualite visuelle

- le statut global est compris en un regard
- l'action principale est evidente sans dominer tout le panneau
- chaque carte display est lisible en etat ON et OFF
- la hierarchie fonctionne meme sans couleur
- les gradients restent un accent, pas une texture de fond globale
- l'interface garde sa clarte en petite fenetre
- la UI reste elegante avec 1, 2 ou 6 displays
- aucun composant n'a besoin d'une animation pour etre compris

### Definition of done design avant build

- une palette stable existe
- les roles typographiques sont definis
- les etats des composants principaux sont listes
- la microcopy de base est coherente
- les anti-patterns visuels sont connus
- la direction est assez claire pour designer puis coder sans repartir de zero
