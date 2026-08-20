# Audit — caches en mémoire vive à l'exécution

Périmètre : `crates/qbz-cache/src/{image_cache,audio_cache,playback_cache}.rs`,
`crates/qbz-offline-cache/`, `crates/qbz-app/src/settings/search_cache.rs`,
`crates/qbz-app/src/settings/favorites_cache.rs`, `crates/qbz/src/fav_cache.rs`.

Lecture seule, 2026-08-20.

## Résumé

| Fichier | RAM ou disque | Borné ? | Taille par défaut |
|---|---|---|---|
| `qbz-cache/audio_cache.rs` (L1) | RAM | Oui — LRU par octets | 400 MB (`AudioCache::default()`), instancié à 400 MB dans `qbz-player/src/player/mod.rs:4005` |
| `qbz-cache/playback_cache.rs` (L2) | Disque | Oui — LRU par octets | 800 MB (instancié dans `qbz-player/src/player/mod.rs:3998`) |
| `qbz-cache/image_cache.rs` | Disque (SQLite + fichiers) | **Partiellement** — `evict()` existe mais n'est appelé **qu'une fois au démarrage** (`spawn_evict`, commentaire "Runs once at startup") | `MAX_CACHE_BYTES` (à vérifier dans `qbz/src/artwork.rs`) |
| `qbz-app/settings/search_cache.rs` — slice volatile (albums/tracks/playlists) | RAM | Oui — LRU par nombre de requêtes | `VOLATILE_CACHE_CAPACITY = 40` requêtes |
| `qbz-app/settings/search_cache.rs` — slice artistes persistée | RAM (HashMap en mémoire, miroir JSON disque) | **Non borné** — commentaire explicite : "Only the volatile maps are bounded; the persisted artist store is not evicted here" | Aucune |
| `qbz-app/settings/favorites_cache.rs` | Disque (SQLite) | N/A — taille = nb réel de favoris de l'utilisateur, pas une donnée de cache qui grossit avec l'usage | N/A |
| `qbz/src/fav_cache.rs` | RAM (4× `HashSet` process-wide : tracks, albums, artists, awards) | N/A — même logique : miroir en RAM des favoris disque, taille bornée par les favoris réels de l'utilisateur | N/A |
| `qbz-offline-cache/` | Disque (téléchargements offline volontaires) | Oui — `limit_bytes` par défaut 5 GB, `check_cache_limit()` bloque les nouveaux téléchargements au-delà | 5 GB (`state.rs:48`), configurable et persisté |

## Détails par fichier

### `qbz-cache/src/audio_cache.rs` — L1 mémoire (RAM), BORNÉ
`AudioCache` maintient un `HashMap<u64, CachedTrack>` + un ordre d'accès LRU, protégé par `Mutex`. `insert()` évince les entrées les plus anciennes tant que `current_size + size > max_size_bytes` (lignes 186-198), avant même d'écrire le nouveau morceau. Taille par défaut : 400 MB (`Default`, commentaire "~4-5 Hi-Res tracks"). Utilisé avec ce même 400 MB dans `qbz-player/src/player/mod.rs`. Les entrées évincées partent vers le L2 disque (`PlaybackCache`) plutôt que d'être perdues — pas de fuite RAM possible sur une session longue.

### `qbz-cache/src/playback_cache.rs` — L2 disque, BORNÉ
Cache disque (`~/.cache/qbz/playback/{track_id}.audio`) avec état en RAM (`HashMap<u64, CacheEntry>` — métadonnées seulement, pas les octets audio). `evict_if_needed()` (ligne 248) fait de l'éviction LRU par `last_accessed` avant chaque écriture. Taille par défaut : 800 MB. L'état RAM lui-même (métadonnées) est proportionnel au nombre de fichiers cachés sur disque, donc indirectement borné par les 800 MB / taille moyenne d'un morceau — pas un risque de croissance RAM significatif.

### `qbz-cache/src/image_cache.rs` — cache disque d'artwork, ÉVICTION PARTIELLE
Cache SQLite + fichiers sur disque (`~/.cache/qbz/images/`), pas une structure RAM per se (la RAM tenue est juste la `Connection` SQLite). `evict(max_bytes)` fait bien du LRU par `last_accessed` (lignes 118-165), MAIS le seul appelant (`spawn_evict` dans `qbz/src/artwork.rs:459`, commentaire ligne 458 : *"Trim the image cache to the size budget. Runs once at startup."*) ne le déclenche **qu'au lancement de l'app**. Sur une session d'écoute longue (plusieurs heures, beaucoup de pochettes chargées), le cache disque continue de grossir via `store()` sans jamais re-déclencher `evict()` — il ne sera retaillé qu'au prochain démarrage. C'est un cache disque, donc pas un risque RAM direct, mais correspond exactement au symptôme "grossit indéfiniment sur une session longue" décrit dans la tâche. À vérifier : la valeur de `MAX_CACHE_BYTES` dans `qbz/src/artwork.rs`.

### `qbz-app/src/settings/search_cache.rs` — RAM, BORNÉ pour le volatil / NON BORNÉ pour les artistes
Deux étages :
- **Volatil** (albums/tracks/playlists) : `HashMap` + `VecDeque` d'ordre d'insertion, LRU strict borné à `VOLATILE_CACHE_CAPACITY = 40` requêtes distinctes (`evict_to_bound()`, ligne 228). Testé (`lru_evicts_oldest_beyond_bound`).
- **Persisté artistes** (`ArtistCacheStore`) : `HashMap<String, Vec<Artist>>` tenu en RAM en permanence, miroir sur disque (`search_artist_cache.json`), et **jamais éligible à l'éviction** — commentaire explicite ligne 227 : *"Only the volatile maps are bounded; the persisted artist store is not evicted here (it is small and survives restarts by design)."* Chaque requête de recherche distincte ajoute une clé qui ne sera plus jamais retirée. Sur une session avec beaucoup de recherches variées (ou au fil des mois d'utilisation puisque c'est persisté), cette table ne fait que croître — designed as "small" mais aucune borne ne l'impose réellement.

### `qbz-app/src/settings/favorites_cache.rs` et `qbz/src/fav_cache.rs` — RAM + disque, hors périmètre "cache qui grossit"
Ce sont des miroirs RAM (`HashSet` process-wide dans `fav_cache.rs`) d'une donnée utilisateur réelle (les favoris/follows Qobuz), pas un cache de résultats qui accumule à chaque interaction. Leur taille est bornée par le nombre réel de favoris de l'utilisateur (quelques centaines à quelques milliers tout au plus), synchronisée en full-replace (`set_all*`) ou en delta (`set`/`set_album`/etc.) avec le disque. Aucun risque de croissance non bornée pendant une session — la taille ne bouge qu'avec les actions explicites de favori/follow de l'utilisateur.

### `qbz-offline-cache/` — disque, BORNÉ par défaut mais volontairement désactivable
`OfflineCacheState::limit_bytes` par défaut = 5 GB (`state.rs:48`, `Some(5 * 1024 * 1024 * 1024u64)`), persisté et rechargeable via `apply_persisted_limit`. `maintenance::check_cache_limit()` est un **garde-fou préventif** (bloque une nouvelle mise en cache si `total_size_bytes >= limit`) et non une éviction automatique — si l'utilisateur pousse la limite à `None` ("illimité"), rien ne la retaille jamais toute seule ; c'est un choix explicite de l'utilisateur (téléchargements offline volontaires), pas une fuite silencieuse. Pas de structure RAM correspondante qui grossirait avec le contenu caché — seulement des métadonnées SQLite (`db.rs`) et connexions.

## Verdict

Deux points concrets identifiés côté "grossit sans qu'on le remarque" :

1. **`image_cache.rs`** : le budget de taille (`evict()`) n'est appliqué qu'au démarrage, pas pendant la session — une session d'écoute longue avec beaucoup de pochettes chargées peut dépasser `MAX_CACHE_BYTES` sur disque jusqu'au prochain redémarrage.
2. **`search_cache.rs` (slice artistes)** : `HashMap` persisté sans aucune borne de taille, contrairement au slice volatil (LRU à 40) — croît indéfiniment avec chaque requête de recherche distincte, à la fois en RAM et sur disque (fichier JSON).

Tout le reste du périmètre (audio_cache RAM 400MB, playback_cache disque 800MB, offline-cache 5GB, favoris) est correctement borné ou n'est pas un cache à proprement parler (données utilisateur réelles).
