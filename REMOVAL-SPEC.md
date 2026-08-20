# Spec de suppression — Paroles, Cast, Qobuz Connect, modes multiples

Décidé le 20/08/2026. Document de préparation uniquement, aucun code touché
— exécution différée jusqu'à confirmation que `main` compile (build GitHub
Actions run 32359676695 en cours). Objectif : réduire la surface de l'appli
(un seul mode de lecture) et couper des pans entiers de dépendances
(`qbz-cast`, les 4 crates `qconnect-*`, `qbz-lyrics`).

## 1. Paroles (Lyrics) — suppression totale

| Fichier | Action |
|---|---|
| `qbz-ui/ui/shell/LyricsSidebar.slint` | Supprimer |
| `qbz-ui/ui/shell/LyricsLinesView.slint` | Supprimer |
| `qbz-ui/ui/shell/LyricsControlsFlyout.slint` | Supprimer |
| `qbz-ui/ui/immersive/ImmersiveLyricsFocusPanel.slint` | Supprimer (le dossier `immersive/` part entier de toute façon, §4) |
| `qbz-ui/ui/miniplayer/MiniLyricsSurface.slint` | Supprimer (le dossier `miniplayer/` part entier de toute façon, §4) |
| Bouton "Lyrics" dans `playerbar/PlayerBarActionButtons.slint` | Retirer le bloc bouton |
| `state.slint` : `export global LyricsState` (ligne ~4499) | Supprimer |
| `crates/qbz/src/lyrics.rs`, `lyrics_sync.rs`, `lyrics_prefs.rs`, `lyrics_measure.rs` | Supprimer + retirer les `mod lyrics*;` dans `main.rs` |
| `crates/qbz-qobuz/src/lyrics.rs` | Supprimer + retirer du client Qobuz |
| `crates/qbz-lyrics/` (crate entier) | Supprimer du workspace (`crates/Cargo.toml` membres) |
| `crates/qbzd/src/api/lyrics.rs`, `crates/qbzd/src/cli/lyrics.rs` | Supprimer (le daemon headless `qbzd` perd la commande lyrics) |

## 2. Cast (Chromecast/DLNA) — suppression totale, PAS Qobuz Connect

| Fichier | Action |
|---|---|
| `qbz-ui/ui/shell/CastPicker.slint` | Supprimer |
| `qbz-ui/ui/assets/icons/cast.svg` | Supprimer |
| Bouton "Cast" dans `playerbar/PlayerBarActionButtons.slint` | Retirer le bloc bouton |
| `state.slint` : `export global CastState` (ligne ~4665) | Supprimer |
| `crates/qbz-cast/` (crate entier) | Supprimer du workspace |
| Sites d'appel `CastState`/`CastActions` dans `main.rs` | Retirer (grep `CastState\|CastActions`) |

## 3. Qobuz Connect — suppression totale (décidé explicitement, distinct de Cast)

| Fichier | Action |
|---|---|
| `qbz-ui/ui/shell/QconnectDevModal.slint` | Supprimer |
| `qbz-ui/ui/shell/playerbar/ConnectButton.slint` | Supprimer |
| `qbz-ui/ui/shell/playerbar/QconnectFlyout.slint` | Supprimer |
| `qbz-ui/ui/shell/playerbar/QconnectStatusHeader.slint` | Supprimer |
| `qbz-ui/ui/shell/playerbar/QconnectDeviceList.slint` | Supprimer |
| `qbz-ui/ui/shell/playerbar/QconnectDeviceRow.slint` | Supprimer |
| `qbz-ui/ui/shell/playerbar/QconnectToggleButton.slint` | Supprimer |
| `qbz-ui/ui/shell/playerbar/PlayerBarSmallQconnectFlyout.slint` | Supprimer |
| Bouton Connect dans `PlayerBarActionButtons.slint`/`PlayerBarSmallActionButtons.slint` | Retirer le bloc bouton |
| `state.slint` : `export global QconnectDevState`, `QconnectDevice` (ligne ~4635) | Supprimer |
| `crates/qconnect-app/`, `qconnect-core/`, `qconnect-protocol/`, `qconnect-transport-ws/` (4 crates) | Supprimer du workspace |
| `crates/qbz/src/qconnect_service.rs` (2756L) | Supprimer + tous les sites d'appel dans `main.rs` |

⚠️ Note pour l'agent d'exécution : `NowPlayingState.qconnect-connected`/`is-remote`
dans `state.slint` semblent aussi liés à Qobuz Connect (le "remote renderer"
qui verrouille le volume, `vol-locked` dans `PlayerBar.slint`) — vérifier si
ces champs sont PUREMENT Qconnect ou partagés avec autre chose avant de les
retirer, sinon `VolumeCluster.slint` casse.

## 4. Modes multiples → un seul mode : Large (`npb-mode == 3`)

**Décision** : le layout Large devient LE layout, plus de sélecteur.
Miniplayer, Immersive et Kiosk disparaissent complètement (fenêtres/profils
séparés, pas des variantes du player bar).

| Cible | Action |
|---|---|
| `qbz-ui/ui/immersive/` (13 fichiers) | Supprimer le dossier entier |
| `qbz-ui/ui/miniplayer/` (7 fichiers, hors Lyrics déjà compté §1) | Supprimer le dossier entier |
| `qbz-ui/ui/shell/Kiosk*.slint` (12 fichiers) | Supprimer |
| `playerbar/NowPlayingViewMenu.slint`, `PlayerBarSmallViewModeMenu.slint` | Supprimer (plus rien à choisir) + retirer le bouton qui les ouvrait |
| `PlayerBar.slint`, `PlayerBarLeftColumn.slint`, `PlayerBarCentreColumn.slint`, `PlayerBarActionButtons.slint`, `PlayerBarRightColumn.slint` | Retirer toutes les branches `if npb-mode == 0/1/2`, ne garder que le contenu de la branche `== 3`. Une fois qu'il n'y a plus qu'une branche, le `if` lui-même disparaît — probablement l'occasion de refusionner certains fichiers, à réévaluer une fois fait (peuvent redescendre sous 130L naturellement) |
| `PlayerBarSmall.slint` + ses sous-fichiers | **Supprimer entièrement** — Small est un des modes retirés, tout le dossier `playerbar/PlayerBarSmall*.slint` (13 fichiers créés aujourd'hui) part |
| `state.slint` : champ `npb-mode` sur `ShellState` | Peut devenir une constante ou disparaître complètement selon si autre chose le lit |
| `state.slint` : `kiosk-profile` sur `ShellState` | Supprimer |
| `qbz-ui/ui/shell/NowPlayingBar.slint`, `AppShell.slint`, `search/Cortinilla.slint` | Retirer leurs branches conditionnelles sur `npb-mode` (vérifier chacune, usages différents du même champ) |
| `crates/qbz/src/main.rs` : `kiosk_profile_active()`, `resolved_shell_screen()`, tout le câblage `"npb-view"` (miniplayer/immersive/toggle-profile) | Supprimer les branches devenues mortes |
| `crates/qbz/src/ui_prefs.rs` | Retirer le champ de préférence npb-mode s'il stocke un choix persistant (sinon la valeur sauvegardée d'un ancien mode pourrait planter au chargement) |

## 5. Ordre d'exécution recommandé

1. §1 (Lyrics) et §2 (Cast) d'abord — les plus isolés, aucune dépendance
   entre eux ni avec §3/§4.
2. §3 (Qobuz Connect) — vérifier la note ⚠️ sur `vol-locked`/`is-remote`
   avant de toucher `state.slint`.
3. §4 (modes) en dernier — le plus large, touche `main.rs` et `state.slint`
   au cœur, et dépend d'avoir déjà retiré les boutons Cast/Connect/Lyrics de
   `PlayerBarActionButtons.slint` (sinon double-retrait qui se marche dessus).
4. `crates/Cargo.toml` : retirer les 6 membres de workspace supprimés
   (`qbz-cast`, `qbz-lyrics`, `qconnect-app`, `qconnect-core`,
   `qconnect-protocol`, `qconnect-transport-ws`) et leurs dépendances
   devenues inutiles (`rust_cast`, `ssdp-client`, `rupnp`, `mdns-sd` — à
   vérifier qu'aucun autre crate ne s'en sert encore avant de les retirer du
   lockfile).
5. Compiler et corriger — c'est le chantier le plus large de la session,
   ne PAS tenter à l'aveugle sans retour de compilation.
