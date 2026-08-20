# Audit — coût de calcul du moteur de recommandations

Périmètre : `crates/qbz-reco/{suggestions,sparse_vector,weights,builder,store}.rs`,
`crates/qbz-external-reco/`. Lecture seule, aucun fichier modifié.

## 1. Tourne en fond ou à la demande ?

**À la demande uniquement — pas de tâche de fond, pas de timer, pas de `tokio::spawn` en boucle.**

- `grep` sur `tokio::spawn|interval|sleep|thread::spawn` dans les deux crates :
  aucun résultat. Le moteur n'a aucune boucle propre.
- `SuggestionsEngine` et `ArtistVectorBuilder` sont instanciés à la volée dans
  `qbz-core/src/core.rs::generate_playlist_suggestions` (ligne ~2084), à chaque appel —
  pas d'instance persistante en mémoire entre deux appels.
- Déclencheurs identifiés :
  - `crates/qbz/src/playlist_suggestions.rs` — section "Suggested Songs" d'une
    playlist ouverte. Fetch initial (`Phase::Initial`) à l'ouverture de la playlist,
    puis `Phase::Merge` en pagination/scroll ("load more", seuil `MIN_AVAILABLE_THRESHOLD=12`).
    Rien ne se déclenche à la lecture d'un morceau.
  - `crates/qbzd/src/api/reco.rs` — endpoint API headless équivalent (daemon).
  - `qbz-external-reco` (Discover "Recommandations") : construit uniquement quand
    l'onglet Discover est ouvert, via `build_external_carousels` / les
    `build_rec_*` individuels — pas de cron interne.

## 2. Taille des structures en mémoire

- **`SparseVector`** (`sparse_vector.rs`) : deux `Vec` parallèles (`indices: Vec<u32>`,
  `values: Vec<f32>`), triés par indice, recherche par `binary_search`. Pas de
  structure fixe de dimension — la taille = nombre de relations (nnz).
- **Par artiste, en pratique** : le vecteur d'un artiste est peuplé par
  `builder.rs::build_from_musicbrainz` (members + past_members + groups +
  collaborators — typiquement quelques unités à quelques dizaines d'entrées par
  artiste, MusicBrainz ne renvoie pas des centaines de relations) et
  `build_from_qobuz` (fixé à **20** similar artists via `get_similar_artists(qobuz_artist_id, 20, 0)`,
  ligne 265 de `builder.rs`). Donc **nnz par artiste ≈ 20–50 entrées** dans le cas
  courant, pas de structure qui grossit sans borne.
- **`ArtistVectorStore`** (`store.rs`) : SQLite persistant (`artist_vectors.db`,
  WAL), pas un cache en RAM géant — seul l'index `mbid ↔ idx` est tenu en mémoire
  (`HashMap<String,u32>` + `Vec<String>`), une entrée par artiste jamais vu, donc
  proportionnel au nombre d'artistes distincts croisés par l'utilisateur au fil du
  temps (potentiellement des milliers sur une longue session d'usage, mais chaque
  entrée est juste une String + un u32 — négligeable, quelques dizaines de Ko même
  à 5-10k artistes). Les vecteurs eux-mêmes (`vector_entries`) restent en base,
  recalculés à la demande via `get_vector` (`SELECT ... GROUP BY target_idx`), pas
  chargés massivement en RAM.
- **Pool de suggestions runtime** (`playlist_suggestions.rs`) : bornes explicites —
  `INITIAL_POOL=30`, `EXPANDED_POOL=100`, `MAX_POOL=200` tracks (`SuggestedTrack`,
  quelques champs `String`/`u64`/`f32` chacun). Négligeable (< 1 Mo même à 200
  entrées avec URLs d'artwork).
- **`qbz-external-reco`** : pas de structures en RAM significatives — cache SQLite
  (`external_reco_cache.db`) avec TTL 7j/30j/48h/9j selon le type d'entrée
  (`cache.rs`), pas de gros vecteur persistant.

Conclusion : aucune structure sparse ou autre ne grossit sans borne ; tout est
plafonné (20 similar/artiste, 200 tracks max en pool) ou déporté sur SQLite.

## 3. Recalcul complet trop fréquent ?

**Non — le design a un TTL explicite pour éviter le recalcul à chaque écoute :**

- `builder.rs::ensure_vector` vérifie `store.has_fresh_vector(mbid, max_age_secs)`
  avant de reconstruire quoi que ce soit. `SuggestionConfig::vector_max_age_days`
  par défaut = **7 jours** (`suggestions.rs` ligne 58) — un vecteur d'artiste n'est
  reconstruit que si absent ou plus vieux que 7 jours, jamais à chaque écoute.
- `store::VECTOR_TTL_SECS` = 7 jours également (cohérent).
- `qbz-external-reco/cache.rs` : TTL différenciés — 30j pour les résolutions
  positives, 7j pour les négatives, **48h** pour le bloc de résultats construit
  ("Recommandations" tab), 9j pour les playlists hebdo ListenBrainz (calé sur leur
  cadence de régénération), avec un fallback "stale" à 21j pour éviter une ligne
  vide en cas de hoquet transitoire — donc pas de rebuild à chaque ouverture de
  l'onglet Discover, seulement toutes les 48h.
- Le seul point qui s'exécute "souvent" est le **matching contre le catalogue
  Qobuz** (`search_artist_tracks`, `validate_qobuz_artist`) à chaque ouverture de
  playlist — mais c'est un appel réseau/API Qobuz de recherche, pas un recalcul du
  moteur de reco lui-même ; le vecteur d'artiste sous-jacent reste caché 7 jours.
- Rien n'est déclenché "à chaque écoute" (per-track-play) : le seul hook lié à la
  lecture serait `user_affinity` dans `weights.rs`, mais aucun code dans les
  fichiers audités ne l'alimente automatiquement à la lecture — c'est un poids
  défini mais son alimentation comportementale n'apparaît pas câblée ici (pas de
  callback "on track played" trouvé dans ce périmètre).

## Fichiers de référence

- `/home/vensam/dev/qbz/crates/qbz-reco/src/suggestions.rs`
- `/home/vensam/dev/qbz/crates/qbz-reco/src/sparse_vector.rs`
- `/home/vensam/dev/qbz/crates/qbz-reco/src/weights.rs`
- `/home/vensam/dev/qbz/crates/qbz-reco/src/builder.rs`
- `/home/vensam/dev/qbz/crates/qbz-reco/src/store.rs`
- `/home/vensam/dev/qbz/crates/qbz-external-reco/src/cache.rs`
- `/home/vensam/dev/qbz/crates/qbz-core/src/core.rs` (ligne ~2084, point d'entrée)
- `/home/vensam/dev/qbz/crates/qbz/src/playlist_suggestions.rs` (déclencheur UI)
