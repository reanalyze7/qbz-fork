# Audit légèreté — `local_library.rs` et `qbz-models` (lecture seule)

Date : 2026-08-20. Aucun fichier modifié, aucune compilation lancée.

## 1. `crates/qbz/src/local_library.rs` (4211 L)

### Top 5 fonctions par taille (mesurée entre débuts de `fn` consécutifs)

| Rang | Fonction | Lignes | Ce qu'elle fait |
|---|---|---|---|
| 1 | `ensure_artists_loaded` (3504-3693, 190L) | Charge la liste Artistes : requête DB locale + **branche Plex complète** (union des albums Plex via ATTACH, agrégation client-side des artistes Plex, fusion de portraits), dédup via `merge_artists`, puis dispatch des jobs d'artwork (HTTP vs local/Plex) et lance le fetch Qobuz des portraits manquants. |
| 2 | `apply_album_version` (1920-2047, 128L) | Bascule l'affichage sur une version alternative d'un album (édition/remaster), reconstruit les lignes de pistes et déclenche la ré-hydratation qualité si Plex est actif. |
| 3 | `spawn_plex_quality_hydration` (2048-2169, 122L) | **100 % Plex** : requête asynchrone au serveur Plex pour obtenir bit-depth/sample-rate réels (le cache Plex ne les connaît qu'après lecture), puis republie les mises à jour de qualité dans l'UI. |
| 4 | `select_folder` (2915-3034, 120L) | Ouvre le détail d'un dossier de la bibliothèque : charge les tracks, sous-dossiers, cover, et prépare l'état "détail dossier". |
| 5 | `open_local_album` (1803-1919, 117L) | Ouvre le détail d'un album local : charge les pistes, groupe par version/disque, déclenche l'artwork et — si Plex actif — la hydratation qualité (#3). |

**Observation transversale** : sur les 5 plus grosses fonctions, 3 sont Plex-only ou Plex-branchées (#1, #3, #5 en appelle un). Le fichier contient aussi tout le pipeline `spawn_plex_quality_hydration` → `refresh_open_album_after_hydration` → `refresh_album_card_quality` → `refresh_track_rows_quality`, `plex_cache_db_path`, et la branche Plex de `fetch_tracks_page`/`sort_tracks_like_sql` (voir plus bas). Le `REMOVAL-SPEC.md` (§6) confirme la suppression décidée de **Plex** (`crates/qbz-plex/`, `plex_settings.rs`, `plex_auth.rs`) mais ne liste **pas** ces sites d'appel dans `local_library.rs` — l'exécution de la suppression Plex devra aussi purger une fraction significative des plus grosses fonctions de ce fichier (grep rapide : ~15+ sites `plex`/`Plex` rien que dans ce fichier, au-delà des 5 citées).

### Chevauchement de responsabilité `qbz/local_library.rs` ↔ `qbz-library/src/*`

Frontière globalement **saine et respectée** :
- Tout accès SQL passe par `crate::library_db::with_db(|db| …)` vers `qbz-library::database.rs` (`get_albums_with_full_filter`, `get_albums_metadata_page`, `search_with_filter_page`, `list_folder_children`, etc.) — aucune requête SQL n'est dupliquée en dur dans `local_library.rs`.
- Les types de données (`LocalTrack`, `LocalAlbum`, `LocalArtist`) sont définis une seule fois, dans `qbz-library/src/models.rs`.

Deux zones méritent néanmoins d'être signalées :

1. **`sort_tracks_like_sql`** (ligne 1123, ~37L) — comparateur Rust qui **réimplique manuellement** la sémantique de l'`ORDER BY` SQL de `search_with_filter_page` (COLLATE NOCASE, NULLs-first, etc.). C'est documenté et justifié (ne sert qu'à re-trier côté client la page 1 quand des lignes Plex sont préfixées hors SQL), mais c'est un point de synchronisation manuelle : si l'`ORDER BY` de `database.rs` change, ce comparateur doit être mis à jour à la main sans garde-fou de compilation. Encore un artefact 100 % Plex qui disparaît avec la suppression Plex.
2. **`album_matches_filters`** (ligne 246) et **`album_matches_artist`** (ligne 3368) — filtrage **client-side** (qualité/format/source, sélection d'artiste) sur les lignes déjà chargées en mémoire, distinct des filtres SQL (`get_albums_with_full_filter`). Ce n'est pas une duplication illégitime : le filtrage SQL sert au chargement/pagination, le filtrage Rust sert à l'interactivité immédiate (cases à cocher) sans aller-retour DB. La frontière est correcte, mais la logique de correspondance format→qualité (lignes 246-283) est un exemple de règle métier qui vit **uniquement** côté UI (`qbz/`) alors qu'elle pourrait légitimement vivre dans `qbz-library` si un jour un autre frontend (headless `qbzd`) doit appliquer le même filtre.

**Conclusion volet 1** : pas de duplication de requêtes/structures entre les deux crates. Le vrai problème de poids du fichier est la présence de **toute la logique d'intégration Plex** (chargement, hydratation qualité, fusion d'artistes, artwork tokenisé) directement dans le fichier UI de la bibliothèque locale, alors que Plex est une fonctionnalité annexe déjà actée pour suppression totale (`REMOVAL-SPEC.md` §6). Sa suppression réduira mécaniquement `local_library.rs` de plusieurs centaines de lignes et fera chuter 3 des 5 plus grosses fonctions.

---

## 2. `crates/qbz-models/src/types.rs` (1528 L) et `crates/qbz-models/src/source.rs` (494 L)

### Champs de fonctionnalités annexes déjà actées pour suppression, embarqués dans des structs partagés

`REMOVAL-SPEC.md` §6 acte la suppression totale de **Awards**, **Purchases**, **Radio**, **Plex**, **LastFM/Discogs/Discord**. Vérification dans `types.rs` :

| Feature à supprimer | Où elle vit dans `types.rs` | Isolée ou embarquée dans un struct partagé ? |
|---|---|---|
| **Purchases** | `PurchaseResponse`, `PurchaseIdsResponse`, `PurchaseAlbum`, `PurchaseTrack`, `PurchaseFormatOption` (lignes 894-1010) | ✅ **Isolée** — structs dédiés, aucun champ `purchase_*` sur `Track`/`Album`/`Artist`. Bonne séparation déjà en place (cohérent avec `REMOVAL-SPEC.md` qui ne cible que `purchase_serde.rs` et les fichiers dédiés). Point d'attention : `PurchaseResponse`/`PurchaseIdsResponse`/`RadioResponse` dépendent de `crate::purchase_serde::lenient_page`/`lenient_page_flexible` — supprimer `purchase_serde.rs` sans traiter `RadioResponse` (qui l'utilise aussi, ligne 1028) casserait la compilation ; le spec ne mentionne pas ce couplage. |
| **Radio** | `RadioResponse` (ligne 1021-1030) | ✅ Isolée, struct dédié. Mais partage `purchase_serde::lenient_page_flexible` (voir ci-dessus). |
| **Awards** | `AwardMagazine`, `AwardPageData`, `AwardPageContainer`, `AwardPageGenericList`, `AlbumAward`, `PageArtistAward` (lignes 703-774, 1312-1360) **+ `deserialize_award_id`/`deserialize_award_awarded_at`** | ⚠️ **Partiellement embarquée** — voir ci-dessous, c'est le vrai problème. |
| **Plex** | `source.rs` : `PlaybackSource::Plex`, `TrackOriginTag::Plex`, `ArtworkRef::PlexThumb { base_url, token, path, size }`, `plex_thumb_url()` | ⚠️ **Embarquée** dans les enums partagés `PlaybackSource`/`ArtworkRef` utilisés par `QueueTrack` (voir ci-dessous). |
| LastFM/Discogs/Discord | absents de `types.rs`/`source.rs` (vivent dans `qbz-integrations`) | ✅ N/A, pas de contamination des modèles partagés. |

### Détail — Awards : champ `awards` sur `Album` (struct la plus utilisée du projet)

```
pub struct Album {                       // ligne 429
    ...
    pub awards: Option<Vec<AlbumAward>>, // ligne 485
    pub goodies: Option<Vec<Goody>>,     // ligne 482
    ...
}
```

`Album` est le struct central : instancié pour chaque résultat de recherche, chaque favori, chaque item de `SearchResultsPage<Album>`, chaque suggestion d'album (`AlbumSuggestResponse`), etc. — y compris les chemins hi-res (`hires`, `maximum_sampling_rate`, `maximum_bit_depth` sont sur ce même struct). `awards` et `goodies` sont deux `Option<Vec<T>>` (24 octets chacun sur la structure elle-même, plus l'allocation heap si `Some`) portés par **chaque instance** d'`Album` alors que seule la page Awards (déjà actée pour suppression) les lit.

Même chose sur `DiscoverAlbum` (ligne 1151-1168, champ `awards`, utilisé par tous les blocs `discover/index` : nouveautés, qobuzissims, most_streamed — pas seulement la page Awards) et sur `AlbumFull`/artiste (`pub awards: Option<Vec<PageArtistAward>>`, ligne ~1284, sur le struct de page artiste).

**`REMOVAL-SPEC.md` §6 ne mentionne PAS ces champs** — sa liste pour Awards se limite à `qbz-ui/ui/award/` et `crates/qbz/src/award.rs`. Si l'exécution suit le spec à la lettre, `Album.awards`, `DiscoverAlbum.awards` et la page artiste continueront de porter ce poids mort indéfiniment. C'est un trou concret à signaler avant exécution du chantier.

### Détail — Plex : `ArtworkRef::PlexThumb` dans l'enum de résolution d'artwork

```
pub enum ArtworkRef {                 // source.rs, ligne 118
    Remote(String),
    LocalFile(String),
    PlexThumb { base_url: String, token: String, path: String, size: Option<u32> },
    Embedded(Vec<u8>),
    None,
}
```

La taille en mémoire d'un enum Rust est dictée par sa **plus grosse variante** + tag. `PlexThumb` (3×`String` + `Option<u32>` ≈ 3×24 + 8 = 80 octets) est actuellement la variante la plus lourde de `ArtworkRef` — plus lourde que `Remote(String)` (24 octets) ou `LocalFile(String)`. Chaque `ArtworkRef` produit (résolution de cover pour la now-playing bar, la queue, MPRIS) paie donc la taille de `PlexThumb` même quand la piste n'a jamais touché Plex. `PlaybackSource::Plex` et `TrackOriginTag::Plex` sont de simples variantes d'enum sans champ (coût nul), donc pas un problème de poids — seul `ArtworkRef::PlexThumb` a un coût mesurable, et il est isolé au fichier `source.rs`, pas dispersé.

**Conclusion volet 2** : la séparation Purchases/Radio est propre (structs dédiés). Le vrai point de poids mort est **Awards**, dont les champs (`awards`, `goodies`) sont directement sur `Album`/`DiscoverAlbum` — les structs les plus instanciés du projet, y compris dans les chemins hi-res et les listes longues (recherche, discover, queue). `REMOVAL-SPEC.md` devrait être complété pour couvrir ces champs avant exécution, sinon la suppression du dossier `award/` laissera le poids mémoire en place. Côté Plex, l'impact structurel est limité à une variante d'enum (`ArtworkRef::PlexThumb`) qui gonfle légèrement chaque résolution d'artwork mais reste localisée et disparaîtra proprement avec la suppression déjà planifiée.
