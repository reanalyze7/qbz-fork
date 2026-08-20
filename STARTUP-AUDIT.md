# Audit du démarrage à froid (cold start) — QBZ

Date : 2026-08-20
Portée : `crates/qbz/src/main.rs` (`fn main`, lignes ~8084 à 22577 — c'est
la totalité du corps de fonction, il n'y a pas d'autre `fn` de niveau
racine après `fn main` dans ce fichier), plus les modules qu'elle appelle
directement en synchrone avant `window.show()`.

Méthode : lecture du code réel, pas de mesure chronométrée (l'hôte ne peut
pas compiler ce crate). Les coûts ci-dessous sont des **estimations**
déduites de la nature de chaque appel (lecture disque, parse JSON, ouverture
SQLite, réseau) — à confirmer par un `RUST_LOG=info` + timestamps si besoin
un jour sur une machine capable de builder.

## Vue d'ensemble : où se trouve vraiment le travail

`fn main()` ne se termine (`window.show()` ligne ~22536, avant l'édit —
inchangée par cet audit) qu'après avoir exécuté **la quasi-totalité des
~14 000 lignes** qui suivent `AppWindow::new()` (ligne 8214) : c'est là que
vivent tous les `window.on_xxx(move || { ... })` qui câblent les
gestionnaires d'événements de l'UI. Point important déjà bien conçu dans le
code existant :

- L'essentiel de ce câblage ne fait **aucune I/O au moment de
  l'enregistrement** — un `window.on_click(move || { ... })` alloue une
  closure et l'attache à un signal Slint, ce qui est de l'ordre de la
  microseconde par closure. Le travail réel (réseau, disque) qu'elles
  contiennent ne s'exécute qu'au clic, pas au boot.
- La restauration de session (`auth::restore_saved_session`), l'auth Qobuz,
  et tout `enter_shell(...)` (chargement du Home, playlists, cache
  favoris…) sont déjà lancés via `tokio_rt.spawn(async move { ... })`
  **avant** que le reste des closures ne soit câblé — donc déjà hors du
  chemin bloquant. `window.show()` ne les attend pas.
- Aucun scan de bibliothèque locale (`local_library::*`) ne s'exécute en
  dehors d'un handler `on_xxx` ou d'un `tokio::spawn` — pas de scan
  synchrone au boot.

Donc le vrai goulot n'est pas "une grosse étape bloquante" façon Tauri, mais
une **accumulation de petites I/O synchrones** dispersées dans le corps de
`fn main()`, exécutées en série, avant `window.show()`. C'est ce que cet
audit trace et corrige là où c'est sûr.

## Chronologie du chemin de démarrage (avant `window.show()`)

| # | Étape | Ligne (approx.) | Nature | Coût estimé | Verdict |
|---|---|---|---|---|---|
| 1 | Preset d'échelle UI (`ui_prefs::load()` + `set_var`) | 8092 | 1 lecture+parse JSON | ~0.1–1 ms | **Garder avant** — `SLINT_SCALE_FACTOR` doit être posé avant la création de fenêtre (winit le lit à la création). |
| 2 | `qbz_log::install("info")` | 8106 | ouverture fichier log | ~1 ms | Garder — tout log ultérieur en dépend. |
| 3 | `deep_link::capture_argv()` | 8118 | lecture `argv`, aucune I/O disque | négligeable | Garder. |
| 4 | `qbz_cast::ensure_crypto_provider()` | 8133 | init process-global, CPU pur | négligeable | Garder — doit précéder tout usage TLS. |
| 5 | Création du runtime Tokio | 8135 | allocation de threads | ~1–5 ms | Garder — nécessaire à tout le reste. |
| 6 | Single-instance guard (D-Bus) | 8143 | IPC D-Bus (Linux) | variable, quelques ms | Garder — doit décider avant de créer une 2ᵉ fenêtre. |
| 7 | `select_slint_backend()` (wgpu/GL/logiciel) | 8175 | sonde GPU + `ui_prefs::load()` | quelques ms à dizaines de ms selon le driver | Garder — le rendu graphique est traité séparément (hors scope ici), mais la sélection doit précéder la fenêtre. |
| 8 | Résolution de langue (`ui_prefs::load()` + `qbz_i18n`) | 8201 | 1 lecture+parse JSON | ~0.1–1 ms | Garder — doit précéder `AppWindow::new()` pour le premier paint dans la bonne langue. |
| 9 | `AppWindow::new()` | 8214 | construction Slint (génère l'arbre de composants) | dizaines de ms (dépend du renderer, hors scope) | Garder — c'est la fenêtre elle-même. |
| 10 | **Bloc de seed UI (thème, appearance, purchases, notifications…)** | 8475–8629 | **~15 lectures+parses JSON répétées du même fichier** `ui_prefs.json` | ~1–2 ms par lecture → **15–30 ms cumulés** | **CORRIGÉ** — voir §Changements. Un seul `boot_prefs = ui_prefs::load()` réutilisé. |
| 11 | Cache MusicBrainz (`MusicBrainzCache::new`) | ~8706 (avant édit) | ouverture SQLite + `PRAGMA WAL` + `CREATE TABLE IF NOT EXISTS` | ~2–10 ms (fichier local, mais I/O disque bloquant + syscalls SQLite) | **CORRIGÉ** — déplacé en tâche de fond (`startup_defer::spawn_musicbrainz_cache`). |
| 12 | Cache image artwork (`ImageCacheService::new` via `artwork::open_cache()`) | ~8740 (avant édit) | ouverture SQLite + `PRAGMA WAL` + `CREATE TABLE IF NOT EXISTS` | ~2–10 ms | **CORRIGÉ** — déplacé en tâche de fond (`startup_defer::spawn_image_cache`). |
| 13 | `settings::SettingsCtx::open()` (audio settings + playback prefs) | ~8738 | **2 ouvertures SQLite supplémentaires** | ~4–20 ms cumulées | **Candidat, non traité** — voir §Candidats. |
| 14 | `offline_mode::start()` + `start_ui_forwarder` | ~8752 | démarre un moniteur de connectivité (pas de blocage) | négligeable | Garder. |
| 15 | ~14 000 lignes de câblage `window.on_xxx(...)` | 8760–22536 | allocations de closures, pas d'I/O directe (sauf lectures `ui_prefs::load()` isolées dans certains blocs de seed non-closures, ex. lignes ~10001–15807 — celles-là sont dans des **closures**, donc exécutées au clic, pas au boot) | quelques ms au total pour l'enregistrement | Garder — c'est le câblage de l'app, pas du travail différable sans refonte massive. |
| 16 | `window.show()` | ~22536 | affichage réel | — | Fin du chemin critique mesuré ici. |

## Changements effectués (sûrs, faible risque)

Tous les changements sont dans un nouveau fichier `crates/qbz/src/startup_defer.rs`
(74 lignes, < 130) + des points d'appel modifiés dans `main.rs` et
`artwork.rs`.

### 1. Cache image artwork différé (`startup_defer::spawn_image_cache`)

`artwork::ImageCache` est déjà `Arc<Mutex<Option<ImageCacheService>>>` —
c'est-à-dire que **tout le code consommateur gère déjà le cas "cache pas
encore ouvert"** (`cached_path_for`, `get`, etc. font tous
`guard.as_ref()?` et retombent sur un miss réseau). Le changement :

- publier immédiatement un handle vide (`Arc::new(Mutex::new(None))`) et
  l'enregistrer via `artwork::set_shared_cache` — donc toutes les closures
  câblées plus loin dans `main()` récupèrent le même `Arc` qu'avant (aucune
  restructuration de leur côté) ;
- ouvrir réellement le SQLite (WAL + `CREATE TABLE`) sur une tâche
  `tokio_rt.spawn`, puis remplir le `Mutex` et lancer l'éviction
  (`spawn_evict`) une fois prêt.

Risque : nul en pratique — dans la fenêtre de quelques ms avant que la
tâche de fond ait fini, un lookup d'artwork rate juste le cache disque et
retombe sur le réseau, exactement le comportement déjà prévu pour un cache
"non disponible".

### 2. Cache MusicBrainz différé (`startup_defer::spawn_musicbrainz_cache`)

Même logique : `AppRuntime::set_musicbrainz_cache` est un simple setter sur
`Mutex<Option<MusicBrainzCache>>`, et le commentaire du code d'origine
dit explicitement que "Failure to open just degrades to direct network
calls — the methods skip the cache when none is set." L'ouverture SQLite
est déplacée telle quelle (logique identique) dans une tâche de fond.

### 3. Déduplication des lectures `ui_prefs::load()` au boot (`main.rs`, ~8475–8629)

Le bloc de seed de l'UI (thème, appearance, purchases, notifications,
langue…) appelait `crate::ui_prefs::load()` **~15 fois de suite**, chaque
appel relisant et re-parsant intégralement le même fichier
`ui_prefs.json` depuis le disque (voir `ui_prefs.rs::load()` — aucun
cache interne). Changement : un seul `let boot_prefs = crate::ui_prefs::load();`
en tête de bloc, puis `boot_prefs.clone()` (ou lecture de champ directe) à
chaque usage.

Risque : nul — c'est une séquence strictement séquentielle de lectures
sans écriture intercalée (aucun `ui_prefs::save()` entre les appels
originaux), donc les ~15 lectures étaient déjà garanties de renvoyer la
même valeur. Les handlers `on_xxx` plus loin dans le fichier (recherche
immersive, réglages Discord RPC, etc.) continuent d'appeler
`ui_prefs::load()` frais à chaque clic — volontairement non touchés, ils
doivent voir l'état courant, pas un instantané du boot.

## Candidats identifiés, NON implémentés (risque ou effort disproportionné)

### `settings::SettingsCtx::open()` — 2 ouvertures SQLite synchrones

`crates/qbz/src/settings.rs::SettingsCtx::open()` ouvre `AudioSettingsState`
et `PlaybackPreferencesState` (2 bases SQLite) de façon synchrone. Contrairement
au cache image/MusicBrainz, **`settings_ctx` n'est pas `Option`-wrappé** —
c'est un `Arc<SettingsCtx>` passé directement à des dizaines de closures
plus loin dans `main()`, qui s'attendent à un store déjà valide (lecture de
volume, préférences audio, etc.). Le différer proprement demanderait de
refaire `SettingsCtx` en `Arc<Mutex<Option<SettingsCtx>>>` (ou équivalent)
et de patcher tous les points de consommation — refactor structurel, pas
une simple bascule vers `tokio_rt.spawn`. **Recommandation : revue humaine**
avant de toucher à ça ; le gain (quelques ms) ne semble pas justifier le
risque de casser un lecteur audio dont les préférences ne seraient pas
encore chargées au premier clic.

### Lectures `ui_prefs::load()` restantes, dispersées ailleurs dans `main()`

Une quinzaine d'appels supplémentaires à `ui_prefs::load()` subsistent,
mais **dispersés dans des fonctions/phases différentes** (sélection du
renderer ligne ~317, création du dossier de cache ligne ~753, chrome macOS
ligne ~7515, etc.) — chacun est un appel isolé pour un besoin ponctuel, pas
une répétition évidente comme le bloc corrigé. Les consolider demanderait
de faire remonter `boot_prefs` à travers plusieurs fonctions séparées
(signatures à changer) pour un gain marginal (quelques ×0.1 ms chacun).
**Recommandation : laisser tel quel**, le rapport coût/risque n'est pas
favorable.

### Sélection du renderer GPU (`select_slint_backend`, ligne ~8175)

Hors scope explicite de cette mission (déjà traité séparément côté rendu
GPU d'après le contexte de la tâche) — non ré-audité ici.

## Ce qui est déjà bien conçu (aucun changement nécessaire)

- Restauration de session / auth Qobuz : déjà en tâche de fond
  (`tokio_rt.spawn`), `window.show()` ne l'attend pas.
- Aucun scan de bibliothèque locale synchrone au boot — tous les appels
  `local_library::*` trouvés dans `main()` sont dans des handlers `on_xxx`
  ou des `tokio::spawn`.
- `enter_shell()` (chargement Home, playlists, cache favoris) tourne déjà
  après affichage de l'écran (via `upgrade_in_event_loop`), avec un usage
  correct de `tokio::task::spawn_blocking` pour l'écriture SQLite du cache
  favoris (`fav_cache::set_all`) — modèle à suivre si `SettingsCtx` est un
  jour différé.

## Fichiers touchés

- `crates/qbz/src/startup_defer.rs` — nouveau, 74 lignes.
- `crates/qbz/src/main.rs` — déclaration du module, remplacement des 2
  ouvertures SQLite synchrones par des appels à `startup_defer::*`,
  déduplication des lectures `ui_prefs::load()` dans le bloc de seed UI.
- `crates/qbz/src/artwork.rs` — extraction de `open_cache_blocking()` (le
  cœur de l'ancien `open_cache()`, réutilisé par `startup_defer`).

Note sur la règle des 130 lignes/fichier : le nouveau fichier
`startup_defer.rs` la respecte. `main.rs` (22 577 lignes, une seule
fonction `fn main`) et `artwork.rs` (~2 500 lignes) sont des fichiers
préexistants déjà très au-delà de cette limite ; les découper aurait été un
refactor massif et à haut risque, hors du périmètre "audit + différés sûrs"
de cette mission — les modifications qui y ont été faites sont volontairement
chirurgicales (quelques lignes chacune) plutôt que de prétexter la règle
pour justifier une réécriture non demandée.
