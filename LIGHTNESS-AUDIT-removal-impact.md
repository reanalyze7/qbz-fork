# Audit d'impact — exécution de REMOVAL-SPEC.md

Lecture seule, 20/08/2026. Comptage `wc -l` réel de chaque fichier/dossier
listé dans `REMOVAL-SPEC.md`, plus vérification des dépendances Cargo qui
deviennent orphelines. Aucun fichier modifié.

## Référentiel actuel (avant suppression)

| Mesure | Valeur |
|---|---|
| Crates dans `crates/Cargo.lock` | 1012 |
| Membres du workspace (`crates/Cargo.toml`) | 37 |
| Lignes Slint totales (`find crates -name "*.slint"`, hors `vendor/`) | 74 322 |
| `crates/qbz/src/main.rs` | 22 760 L (référence, non touché) |

## §1 — Paroles (Lyrics)

| Fichier | Lignes |
|---|---|
| `qbz-ui/ui/shell/LyricsSidebar.slint` | 361 |
| `qbz-ui/ui/shell/LyricsLinesView.slint` | 436 |
| `qbz-ui/ui/shell/LyricsControlsFlyout.slint` | 252 |
| `qbz-ui/ui/immersive/ImmersiveLyricsFocusPanel.slint` | 154 |
| `qbz-ui/ui/miniplayer/MiniLyricsSurface.slint` | 46 |
| `crates/qbz/src/lyrics.rs` | 750 |
| `crates/qbz/src/lyrics_sync.rs` | 422 |
| `crates/qbz/src/lyrics_prefs.rs` | 395 |
| `crates/qbz/src/lyrics_measure.rs` | 291 |
| `crates/qbz-qobuz/src/lyrics.rs` | 573 |
| `crates/qbzd/src/api/lyrics.rs` | 92 |
| `crates/qbzd/src/cli/lyrics.rs` | 86 |
| `crates/qbz-lyrics/` (crate entier, 9 fichiers, `.rs` uniquement) | 3 665 |
| **Sous-total Slint** | **1 249** |
| **Sous-total Rust** | **6 274** |
| **Total §1** | **7 523** |

Bouton "Lyrics" (`PlayerBarActionButtons.slint`) et `export global LyricsState`
(`state.slint`) : retraits partiels, non quantifiables par fichier entier
(non comptés ci-dessus).

## §2 — Cast (Chromecast/DLNA)

| Fichier | Lignes |
|---|---|
| `qbz-ui/ui/shell/CastPicker.slint` | 361 |
| `qbz-ui/ui/assets/icons/cast.svg` | 6 (icône, hors code) |
| `crates/qbz-cast/` (crate entier, 11 fichiers, `.rs`) | 2 638 |
| **Total §2 (Slint + Rust)** | **2 999** |

## §3 — Qobuz Connect

| Fichier | Lignes |
|---|---|
| `qbz-ui/ui/shell/QconnectDevModal.slint` | 152 |
| `qbz-ui/ui/shell/playerbar/ConnectButton.slint` | 56 |
| `qbz-ui/ui/shell/playerbar/QconnectFlyout.slint` | 70 |
| `qbz-ui/ui/shell/playerbar/QconnectStatusHeader.slint` | 61 |
| `qbz-ui/ui/shell/playerbar/QconnectDeviceList.slint` | 67 |
| `qbz-ui/ui/shell/playerbar/QconnectDeviceRow.slint` | 64 |
| `qbz-ui/ui/shell/playerbar/QconnectToggleButton.slint` | 42 |
| `qbz-ui/ui/shell/playerbar/PlayerBarSmallQconnectFlyout.slint` | 76 |
| `crates/qbz/src/qconnect_service.rs` | 2 756 |
| `crates/qconnect-app/` (13 fichiers, `.rs`) | 6 175 |
| `crates/qconnect-core/` (8 fichiers, `.rs`) | 1 120 |
| `crates/qconnect-protocol/` (10 fichiers, `.rs`) | 4 196 |
| `crates/qconnect-transport-ws/` (5 fichiers, `.rs`) | 1 467 |
| **Sous-total Slint** | **588** |
| **Sous-total Rust** | **15 714** |
| **Total §3** | **16 302** |

## §4 — Modes multiples → Large seul

| Cible | Lignes | Note |
|---|---|---|
| `qbz-ui/ui/immersive/` (dossier entier) | 4 754 | 4 908 total − 154 (déjà comptées §1) |
| `qbz-ui/ui/miniplayer/` (dossier entier) | 958 | 1 004 total − 46 (déjà comptées §1) |
| `qbz-ui/ui/shell/Kiosk*.slint` | 2 491 | |
| `qbz-ui/ui/shell/playerbar/PlayerBarSmall*.slint` | 868 | |
| `NowPlayingViewMenu.slint` | 73 | |
| `PlayerBarSmallViewModeMenu.slint` | 101 | |
| **Total §4** | **9 245** | 100 % Slint |

Branches `if npb-mode == 0/1/2` dans `PlayerBar*.slint`, câblage `main.rs`
(`kiosk_profile_active`, `resolved_shell_screen`, `"npb-view"`) et champ
`ui_prefs.rs` : retraits partiels dans des fichiers qui survivent, non
quantifiables par fichier entier.

## §6 — Fonctionnalités annexes

| Fonctionnalité | Fichiers | Lignes |
|---|---|---|
| **Plex** | `qbz-plex/src/lib.rs` (2 057) + `qbz-app/src/settings/plex.rs` (723) + `qbz/src/plex_settings.rs` (122) + `qbz/src/plex_auth.rs` (1 030) + `plex-logo.svg` (6) | 3 938 |
| **LastFM/Discogs/Discord/remote_metadata** | `lastfm/` (3 fichiers, 1 082) + `remote_metadata/` (2 fichiers, 625) + `discord.rs` (210) + `discogs/mod.rs` (626) | 2 543 |
| **Radio** | `qbz-radio/` crate (6 fichiers, 1 077) + `RadioCarousel.slint` (119) + `RadioCard.slint` (124) + `radio.svg` (7) | 1 327 |
| **Purchases** | `purchases.rs` (3 184) + `purchases_service.rs` (1 269) + `qbz-qobuz/src/purchases.rs` (485) + `purchase_serde.rs` (336) + `PurchasesView.slint` (1 549) + `PurchaseDetailView.slint` (774) | 7 597 |
| **Awards** | `AwardAlbumsView.slint` (364) + `AwardView.slint` (346) + `award.rs` (642) | 1 352 |
| **Discography Builder** | `DiscographyBuilderView.slint` | 787 |
| **Total §6** | | **17 544** |

## Total lignes de code retirées (Slint + Rust, fichiers entiers uniquement)

| Section | Slint | Rust | Total |
|---|---|---|---|
| §1 Lyrics | 1 249 | 6 274 | 7 523 |
| §2 Cast | 361 | 2 638 | 2 999 |
| §3 Qobuz Connect | 588 | 15 714 | 16 302 |
| §4 Modes | 9 245 | 0 | 9 245 |
| §6 Annexes | 4 063 | 13 468 | 17 531 |
| **Total** | **15 506** | **38 094** | **53 600** |

+ 19 lignes de SVG (icônes `cast.svg`, `plex-logo.svg`, `radio.svg`, hors code).

Ne sont **pas** inclus dans ce total (retraits partiels dans des fichiers qui
restent, non mesurables par `wc -l` d'un fichier entier) : blocs boutons
dans `PlayerBarActionButtons.slint`/`PlayerBarSmallActionButtons.slint`,
globals `state.slint` (`LyricsState`, `CastState`, `QconnectDevState`,
`QconnectDevice`, champ `npb-mode`, `kiosk-profile`), branches `npb-mode`
dans `PlayerBar*.slint`/`NowPlayingBar.slint`/`AppShell.slint`/`Cortinilla.slint`,
sites d'appel dans `main.rs`, section `IntegrationsSettings.slint`/
`SandboxSettings.slint` (LastFM/Discogs/Discord). Le vrai total sera donc
**légèrement supérieur** à 53 600 L une fois ces retraits ponctuels faits.

## Comparaison aux références

| Mesure | Avant | Retiré | Après (estimé) | % |
|---|---|---|---|---|
| Lignes Slint | 74 322 | 15 506 | ~58 816 | −20,9 % |
| Lignes Rust (crates concernées) | — | 38 094 | — | — |
| Membres workspace | 37 | 8 | 29 | −21,6 % |

## Crates workspace supprimées (8)

`qbz-cast`, `qbz-lyrics`, `qconnect-app`, `qconnect-core`, `qconnect-protocol`,
`qconnect-transport-ws`, `qbz-plex`, `qbz-radio`.

(La spec §5 n'en cite explicitement que 6 pour le nettoyage `crates/Cargo.toml`
— `qbz-plex` et `qbz-radio` sont ajoutées par §6 et doivent suivre le même
retrait de membre.)

## Dépendances directes orphelines (vérifiées)

Dépendances déclarées dans les `Cargo.toml` des 8 crates supprimées, croisées
par `grep -l "^<dep>"` contre tous les autres `crates/*/Cargo.toml` :

| Dépendance | Déclarée dans | Utilisée ailleurs dans le workspace ? | Statut |
|---|---|---|---|
| `rust_cast` | qbz-cast | Non | **Orpheline confirmée** (déjà connue) |
| `rupnp` | qbz-cast | Non | **Orpheline confirmée** (déjà connue) |
| `mdns-sd` | qbz-cast | Non | **Orpheline confirmée** (déjà connue) |
| `prost` | qconnect-protocol, qconnect-transport-ws | Non | **Orpheline confirmée** (déjà connue) |
| `tokio-tungstenite` | qconnect-transport-ws | Non | **Orpheline confirmée** (déjà connue) |
| `rustls` (pin direct 0.23, hors `reqwest` feature) | qbz-cast | Non — seul pin direct du repo ; les autres crates n'utilisent `rustls` que via la feature `rustls-tls` de `reqwest` | **Orpheline confirmée (nouvelle)** |
| `getrandom` | qbz-cast | Non | **Orpheline confirmée (nouvelle)** |
| `tiny_http` | qbz-cast | Oui — `crates/qbzd/Cargo.toml` | Reste (qbzd la garde) |
| `urlencoding` | qbz-lyrics | Oui — qbz, qbzd, qbz-integrations, qbz-media-controls, qbz-models | Reste |
| `uuid` | qconnect-app, qconnect-protocol | Oui — qbz-app, qbz, qbzd, qbz-library, qbz-mixtape, qbz-secrets | Reste |
| `dirs` | qbz-plex, qbz-radio | Oui — 9 autres crates | Reste |
| `open` | qbz-plex | Oui — qbz, qbzd | Reste |
| `reqwest` | qbz-lyrics, qbz-plex | Oui — 10 autres crates | Reste |
| `rusqlite` | qbz-lyrics, qbz-plex, qbz-radio | Oui — dépendance quasi universelle du workspace | Reste |
| `serde`/`serde_json`/`tokio`/`thiserror`/`async-trait`/`futures-util`/`log`/`regex` | plusieurs crates supprimées | Oui — `workspace.dependencies`, utilisées partout | Reste |

**7 dépendances directes deviennent totalement orphelines** :
`rust_cast`, `rupnp`, `mdns-sd`, `prost`, `tokio-tungstenite`, `rustls`, `getrandom`
(présentes 1 fois chacune dans `crates/Cargo.lock`, sauf `getrandom` à 3
versions). Leur fermeture transitive (dépendances de ces dépendances, ex.
`tungstenite`, `prost-derive`, les libs mDNS/UPnP internes à `rupnp`/`mdns-sd`)
n'a pas été comptée précisément ici — un `cargo tree -e no-dev` post-retrait
donnerait le chiffre exact, mais elle est notable : `prost` et
`tokio-tungstenite` seuls tirent chacun plusieurs sous-crates (codegen,
protobuf runtime, websocket framing/TLS).
