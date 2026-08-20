# Audit — coût de la génération auto-thème (qbz-theme)

Périmètre lu : `crates/qbz-theme/src/auto/generator.rs` (500L), `auto/system.rs`
(673L), `auto/palette.rs` (309L), `registry.rs` (1851L). Lecture seule, aucune
modif, aucune compilation.

## 1. Déclencheurs — PAS à chaque changement de morceau

L'"auto-theme" n'a **rien à voir avec la pochette d'album en cours de lecture**.
C'est un thème dérivé soit du fond d'écran système (DE : GNOME/KDE/COSMIC/XFCE/
Cinnamon, via `auto/system.rs`), soit d'une image choisie manuellement par
l'utilisateur (`AutoSource::Image`), soit du color-scheme système (KDE/GNOME).
Aucun code ne relie ça à la piste en cours — confirmé par grep (`track_change`,
`now_playing`, `current_track` : aucun hit croisé avec le thème ; aucune notion
de "cover"/pochette dans `auto/*`).

Points d'entrée réels (`crates/qbz/src/auto_theme.rs` + `main.rs`) :
- `apply_startup()` — une fois au lancement, sur le fil événementiel (bloquant,
  volontairement : "le premier paint est déjà la palette générée").
- `regenerate()` — off-thread (`spawn_blocking`), appelé uniquement sur :
  - sélection du thème "Auto (dynamic)" dans le dropdown Settings
  - changement de source (System/Wallpaper/Custom Image)
  - sélection d'une nouvelle image via le picker
  - clic explicite sur le bouton "Regenerate"
- Pas de file-watcher sur le fond d'écran (commentaire explicite en tête de
  fichier : "there is no live wallpaper file-watcher" — déviation assumée vs la
  version Tauri qui régénérait réactivement).

**Conclusion : coût nul en lecture normale.** Zéro décodage d'image, zéro
k-means pendant l'écoute d'un album — seulement sur action utilisateur explicite
dans les réglages d'apparence.

## 2. Coût par génération (quand elle a lieu)

Pipeline (`auto/palette.rs::extract_palette`) :
1. Décodage image borné (garde anti decompression-bomb : max 12000x12000,
   512 Mo max avant traitement).
2. Downsample 100x100 (Lanczos3) → au plus 10 000 pixels.
3. K-means (k=6, max 25 itérations) sur ces pixels → 6 clusters dominants.
4. Assignation des rôles (bg/accent/status) + ajustements de contraste WCAG
   (`generator.rs`, boucle de 20 itérations max par ajustement de luminosité).

Coût borné et raisonnable pour un run ponctuel (image 100x100, k=6) — mais tout
ceci tourne à chaque `regenerate()`, y compris pour re-choisir la même source
sans qu'aucun résultat ne soit mémorisé (voir §3).

## 3. Aucun cache

Aucune trace de cache (pas de `HashMap`/mémoïsation par chemin d'image ou par
hash de pixels, aucun stockage de palette dérivée). Chaque appel à
`regenerate()` relit entièrement l'image/le wallpaper et refait tout le calcul
k-means + contraste, même si :
- la même image/wallpaper a déjà été traitée dans une session précédente
- l'utilisateur clique "Regenerate" sans rien changer

Impact pratique limité car les déclencheurs sont rares (actions explicites dans
Settings), mais un cache clé sur `(source, chemin/hash image)` éviterait un
recalcul identique si l'utilisateur va-et-vient entre thèmes dans les réglages
ou relance "Regenerate" par erreur. Non implémenté aujourd'hui.

## 4. Pourquoi registry.rs fait 1851 lignes

Pas de code mort ni de duplication accidentelle : le fichier matérialise en
dur, un token à la fois, **~30 thèmes statiques** (Dark, Oled, TokyoNight,
Light, Warm, Nord, Dracula, 4 Catppuccin, BreezeDark, AdwaitaDark, Aurora,
Ikari, Ayanami, Iscariot, Stratego, Rumi, Zoey, Mira, Frost, Langley, Alucard,
RosePineDawn, …) — un `fn nom_theme() -> ThemeColors { .. }` par thème, chacun
~20-40 lignes (une valeur par champ `ThemeColors` : surfaces, textes, accent,
danger/warning/success avec leurs teintes bg/border/hover, bordures, alpha
ramp). Design assumé : **pas de cascade CSS côté Slint**, donc chaque thème
doit porter TOUTES ses valeurs explicitement (contrairement à Tauri/CSS où un
thème pouvait omettre des tokens et hériter de `:root`).

Répartition :
- Lignes 1–~1313 : les 30 fonctions de construction de thème + `palette(id)`
  (dispatcher) + constantes legacy partagées.
- Lignes 1314–1851 (≈538 lignes, ~29 % du fichier) : `#[cfg(test)] mod tests`
  — vérifications systématiques (contraste WCAG, cohérence des teintes, etc.)
  par thème. Deux blocs `#[cfg(test)]` (1314 et 1553).

La taille est donc directement proportionnelle au nombre de thèmes livrés
(~30) × (données de thème + tests dédiés), pas un signe de mauvaise
structuration — même si, avec 30 fonctions quasi-identiques dans un seul
fichier, un futur découpage par famille de thèmes (ex. `registry/dark.rs`,
`registry/branded.rs`, `registry/catppuccin.rs`) réduirait la taille du fichier
sans changer le comportement.

## Fichiers consultés

- `crates/qbz-theme/src/auto/generator.rs`
- `crates/qbz-theme/src/auto/system.rs`
- `crates/qbz-theme/src/auto/palette.rs`
- `crates/qbz-theme/src/registry.rs`
- `crates/qbz/src/auto_theme.rs`
- `crates/qbz/src/main.rs` (points d'appel `auto_theme::regenerate` / `apply_startup`)
