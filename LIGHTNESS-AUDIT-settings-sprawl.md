# Audit — `crates/qbz-app/src/settings/` (lecture seule)

21 fichiers, 11 112 lignes. `mod.rs` (21 L) n'est qu'une liste de `pub mod` — pas de sprawl là.

## Inventaire (lignes, `wc -l`)

| Fichier | Lignes |
|---|---|
| mod.rs | 21 |
| daemon_prefs.rs | 122 |
| developer.rs | 213 |
| search_service.rs | 241 |
| subscription.rs | 299 |
| local_favorites.rs | 307 |
| pinned_items.rs | 323 |
| album_play_history.rs | 332 |
| graphics.rs | 338 |
| favorites.rs | 402 |
| search_cache.rs | 412 |
| tray.rs | 416 |
| playback.rs | 452 |
| scrobblers.rs | 462 |
| search_ranking.rs | 486 |
| favorites_cache.rs | 541 |
| artist_blacklist.rs | 667 |
| discover_prefs.rs | 718 |
| plex.rs | 723 |
| remote_control.rs | 729 |
| reco_store.rs | 1339 |
| bundle.rs | 1569 |

## Fichiers lus en détail

`mod.rs`, `daemon_prefs.rs`, `local_favorites.rs`, `pinned_items.rs` (intégral) ; en-têtes (~25-40 L) de `artist_blacklist.rs`, `graphics.rs`, `tray.rs`, `favorites_cache.rs`, `album_play_history.rs`, `search_cache.rs`, `bundle.rs`, `reco_store.rs`, `plex.rs`, `discover_prefs.rs`, `search_ranking.rs`, `playback.rs`, `scrobblers.rs`, `developer.rs`, `subscription.rs`, `remote_control.rs`, `search_service.rs`.

## Constat : deux patterns dupliqués, explicitement documentés comme des copies

### Pattern A — "keyed set store" (SQLite, `HashSet<(kind,id)>` en RAM, pin/unpin ou favorite/unfavorite)

`pinned_items.rs` et `local_favorites.rs` sont quasi-identiques ligne à ligne : même pragma WAL, même schéma `(kind, id)` PRIMARY KEY + CHECK, même `RwLock<HashSet<(String,String)>>`, mêmes méthodes `new`/`new_in_memory`/`init_schema`/`load_from_db`/`is_*`/`pin|favorite`/`unpin|unfavorite`/`list`/`count`/`keys_snapshot`, même style d'erreur `format!("Failed to … : {}", e)`, mêmes tests de lifecycle. Le commentaire en tête de `local_favorites.rs` le dit lui-même : *« Mirrors `pinned_items.rs` (same pragmas, error style, in-memory `(kind, id)` set) »*. `pinned_items.rs` dit à son tour qu'il mirrors `artist_blacklist.rs`.

`artist_blacklist.rs` (667 L) et `favorites_cache.rs` (541 L) suivent la même charpente mais avec plus de tables/colonnes (métadonnées additionnelles, feature flag, deux entités blacklistées) — variantes du même mécanisme, pas des domaines différents.

→ Candidat clair à un mécanisme générique unique : un `KeyedSetStore<T>` paramétré par nom de table/fichier DB, schéma des colonnes additionnelles, et contrainte CHECK sur `kind`/`source`, au lieu de 4 réimplémentations de `load_from_db`/`is_*`/`keys_snapshot`.

### Pattern B — "single-row SQLite settings struct" (une ligne `id=1 CHECK`, struct sérialisée en colonnes, load whole / save whole)

`graphics.rs`, `tray.rs`, `developer.rs`, `playback.rs`, `plex.rs`, `scrobblers.rs`, `remote_control.rs` partagent la même charpente : `struct XSettings { … }` + `impl Default` + `struct XSettingsStore { conn: Connection }` + table singleton `CHECK(id=1)` + `new`/`new_at`/`load`/`save`. Les commentaires se citent mutuellement : `plex.rs` dit *« Mirrors the `super::tray` store: a single-row SQLite table »*, `scrobblers.rs` dit *« Mirrors the `super::plex` store »*. La taille varie (213 à 729 L) uniquement parce que le nombre de champs et la logique métier annexe (ex. `subscription.rs` : grace period ; `remote_control.rs` : origins/tokens) diffère — pas parce que le mécanisme de persistance diffère.

→ Deuxième candidat à un mécanisme générique : un `SingletonSettingsStore<T: Serialize+Deserialize+Default>` avec le nom de table/fichier en paramètre, remplaçant N `load`/`save`/`init_schema` presque identiques.

### Pattern C (mineur, 2-3 variantes) — struct JSON persistée hors SQLite

`daemon_prefs.rs` (JSON direct sur disque, dégradation propre si fichier absent/corrompu) et `search_ranking.rs` / `discover_prefs.rs` (JSON blob, soit fichier soit colonne SQLite) partagent le même contrat « load dégrade en défaut, jamais de panic, save best-effort loggé » mais avec des cibles de sérialisation différentes (fichier vs SQLite). Duplication plus légère que A/B, mais même famille d'idée.

## Fichiers qui NE sont PAS du sprawl — vrais domaines avec logique propre

- **`bundle.rs`** (1569 L) : moteur d'export/import/portabilité (`export`/`plan`/`apply`), orchestrateur transversal qui *consomme* les autres stores (`daemon_prefs`, `playback`, `scrobblers`, `qbz_audio::AudioSettingsStore`). Complexité justifiée par les règles de classification §3/§5 et les tests TDD inline — pas un load/save dupliqué.
- **`reco_store.rs`** (1339 L) : event store + scoring/training (décroissance temporelle, `train()`), logique ML-ish distincte, port cleanroom d'un module Tauri équivalent.
- **`album_play_history.rs`** (332 L) : event log + agrégation `COUNT(*) GROUP BY`, différent des stores clé-valeur (c'est un historique d'événements, pas un ensemble de préférences).
- **`search_cache.rs`** (412 L) : cache LRU en mémoire à deux niveaux de volatilité + persistance JSON partielle — mécanisme distinct (SWR), pas un simple load/save.
- **`search_service.rs`** (241 L) : façade fine qui compose `search_cache` + `search_ranking`, volontairement non générique (documenté). Pas de duplication, juste petit.
- **`subscription.rs`** : struct simple mais logique métier propre (grace period 30 jours) au-dessus du load/save.

## Résumé

Sur 21 fichiers, **au moins 11** (`pinned_items`, `local_favorites`, `artist_blacklist`, `favorites_cache` pour le pattern A ; `graphics`, `tray`, `developer`, `playback`, `plex`, `scrobblers`, `remote_control` pour le pattern B) réimplémentent, chacun séparément, le même mécanisme générique de persistance (schéma SQLite quasi identique, mêmes pragmas, mêmes signatures `load`/`save`/`new_at`), et les commentaires du code le confirment explicitement ("mirrors X"). Deux génériques (`KeyedSetStore<T>` et `SingletonSettingsStore<T>`) remplaceraient l'essentiel de ces ~4200 lignes par des structs de config + un mécanisme partagé. `bundle.rs`, `reco_store.rs`, `album_play_history.rs`, `search_cache.rs` et `search_service.rs` (~3900 L) portent une vraie logique distincte et ne sont pas concernés.
