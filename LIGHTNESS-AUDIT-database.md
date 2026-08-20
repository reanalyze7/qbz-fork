# Audit lecture seule — `qbz-library` : scan, database, metadata

Périmètre : `crates/qbz-library/src/database.rs` (6741 L), `scan.rs` (388 L),
`metadata.rs` (1321 L). Aucun fichier modifié, aucune compilation lancée.

## 1. Le scan bloque-t-il l'UI ?

Non — `scan_with_progress()` (`scan.rs:142`) est une fonction 100 % synchrone
(pas d'`async`, pas d'`.await`, I/O disque + SQLite bloquants), **mais** son
unique appelant l'enveloppe correctement :

```rust
// crates/qbz/src/local_library_settings.rs:701
handle.spawn_blocking(move || {
    ...
    let _ = crate::library_db::with_db(|db| {
        qbz_library::scan_with_progress(db, ids_ref, &artwork_cache, &cancel, &sink)
    });
    ...
});
```

La progression remonte à l'UI via un callback `on_event` throttlé
(`throttle_ok`) + `upgrade_in_event_loop`, donc pas de spam de la boucle
Slint. Annulation gérée par un `AtomicBool` vérifié à chaque frontière de
fichier. **Conclusion : le scan tourne bien en tâche de fond, correctement
isolé du thread UI.**

Point d'attention (convention, pas un bug observé) : `LibraryDatabase` est
une API bloquante ordinaire (`rusqlite` synchrone) — rien au niveau du type
n'empêche un futur appelant d'oublier `spawn_blocking`. Le commentaire en
tête de `crates/qbz/src/folders.rs` documente explicitement la règle :
« All ops are blocking (they open the DB), so async callers wrap them in
`tokio::task::spawn_blocking`. » J'ai vérifié deux agrégations multi-appels
(`playlist_manager.rs:154`, `sidebar.rs:206`) qui respectent bien la
convention. Sur 132 call-sites de `library_db::with_db(...)` recensés dans
`crates/qbz/src/`, seule une fraction est visiblement enveloppée dans le
même fichier (`folders.rs` : 24 `spawn_blocking` pour 132 appels au total
dans l'arbre) — cohérent avec le fait que beaucoup de ces appels sont de
petites lectures/écritures ponctuelles (quelques ms sur SQLite local), pas
forcément un problème réel, mais ce n'est pas *garanti* par la structure du
code — juste par discipline des auteurs.

## 2. Index SQLite — recherche/tri fréquents indexés ?

Schéma (`database.rs:63-260`) : `local_tracks` a des index sur `artist`,
`album`, `album_artist`, `file_path`, `title`, plus un index composite
`(album, album_artist, artist)` dédié au regroupement d'albums. Les tables
annexes (playlists, dossiers, artwork, purchases) ont aussi leurs index sur
les colonnes de jointure/tri (`qobuz_playlist_id`, `position`, `is_hidden`,
`fetched_at`, `album_id`...). **Le schéma est correctement indexé pour les
accès par égalité/tri exact.**

Deux limites réelles, toutes deux dans les chemins de recherche texte :

- **`search_with_filter_page()`** (`database.rs:2923`) et
  **`get_albums_metadata_page_inner()`** (`database.rs:2118`, recherche
  d'albums) construisent un motif `LIKE '%query%'` (wildcard des deux
  côtés). Un index B-tree standard ne peut pas servir ce genre de motif —
  c'est un scan complet de `local_tracks` (ou de l'agrégat) à chaque frappe
  de recherche. Pas de table virtuelle FTS5 dans le schéma. Pour une
  bibliothèque de quelques dizaines de milliers de pistes ça reste
  probablement sous la dizaine de ms sur SQLite, mais c'est le point qui
  dégraderait le premier si la bibliothèque grossit (relecture/CD sans
  perte, HD compilations...).
- Les motifs `file_path LIKE 'prefix%'` (préfixe ancré, ex. filtrage par
  dossier — `database.rs:1210, 1244, 1354, 1578...`) sont en revanche
  **indexables** par `idx_tracks_file_path` (SQLite sait utiliser un index
  B-tree pour un `LIKE` sans wildcard en tête), donc pas de souci ici.

**Pas de scan de table complet caché derrière un accès qui semble indexé** —
les seuls scans complets identifiés sont ceux, attendus, de la recherche
plein-texte par sous-chaîne.

## 3. Les 5 plus grosses fonctions de `database.rs`

| Lignes | Début | Fonction | Ce qu'elle fait |
|---|---|---|---|
| 549 | L289 | `run_migrations()` | Séquence de migrations additives (`ALTER TABLE ... ADD COLUMN`, création d'index/tables) déclenchées par des `pragma_table_info` / `sqlite_master` checks. Chaque bloc = un pattern répété (test existence colonne → si absente, `execute_batch`). C'est de la **complexité justifiée mais répétitive** : bon candidat à factoriser (une petite fonction `add_column_if_missing(table, col, ddl)`) plutôt qu'à laisser copier-coller, mais ce n'est pas un problème de correction. |
| 271 | L2118 | `get_albums_metadata_page_inner()` | Requête paginée d'albums groupés : construit dynamiquement `ORDER BY` (allowlist validée, pas d'injection), filtre réseau, filtre Qobuz, `UNION` optionnel avec une base Plex `ATTACH`ée, pagination avec `COUNT(*) OVER()`. Complexité réelle et assumée (pagination + tri + union multi-source) — mais c'est un cas d'école pour scinder en sous-fonctions (construction de clause WHERE, construction de la partie Plex, exécution) pour rester sous 130 L/fonction si le style du projet le veut. |
| 202 | L63 | `init_schema()` | Un unique `execute_batch` avec tout le DDL (12+ tables, ~20 index). Taille justifiée : c'est littéralement le schéma complet en un bloc SQL, pas de la logique. |
| 166 | L1337 | `get_albums_with_full_filter()` | Variante non paginée de la requête d'albums filtrés — même famille de complexité que `get_albums_metadata_page_inner` (tri/filtre réseau/Qobuz), dupliquée plutôt que réutilisée. Risque de dérive : deux implémentations proches à maintenir en parallèle. |
| 153 | L1855 | `get_albums_metadata_grouped()` | Encore une variante de la même famille (grouping/agrégation d'albums) sans pagination. |

**Verdict global sur `database.rs`** : la taille du fichier (6741 L) vient
surtout du **nombre de fonctions** (schéma + migrations + CRUD sur ~10
tables : pistes, playlists, dossiers, artwork, purchases, Plex...), pas
d'une poignée de fonctions monstrueuses. Les 5 plus grosses sont
individuellement défendables (migrations séquentielles, DDL complet,
requêtes de pagination/tri/union multi-source). Le vrai signal
d'amélioration structurelle n'est pas « refactorer une fonction géante »
mais : (a) **3 fonctions quasi-jumelles** pour la liste d'albums
(`get_albums_metadata_page_inner`, `get_albums_with_full_filter`,
`get_albums_metadata_grouped`) qui pourraient partager un constructeur de
clause commun, et (b) le pattern de migration répété dans
`run_migrations()` qui gagnerait à être une petite fonction utilitaire.

## 4. `metadata.rs` — complexité justifiée ?

Fonctions les plus grosses : `extract_with_roots()` (167 L, `L495`, le
pipeline principal d'extraction de tags — probe du fichier, primary tag
avec repli sur chaque tag secondaire pour combler les métadonnées
manquantes, cas DSD séparé via `extract_dsd()`), `find_folder_artwork()`
(91 L), `extract_dsd()` (75 L, gère DSF/DFF qui ne sont pas lisibles par
`lofty` et nécessitent un démuxage dédié via `qbz_dsd`).

Cette complexité est **réellement liée au domaine** : les commentaires en
ligne (ex. `L551-568`) documentent des bugs réels corrigés (#447 dossiers
nommés comme albums quand le tag primaire manque l'album ; #507
`album_artist` NULL sur les tags secondaires) — c'est du code qui a grossi
pour gérer des bibliothèques mal taguées dans la vraie vie, pas de
l'accumulation arbitraire. Pas de red flag ici.

## Résumé

- **Scan** : bien en tâche de fond (`spawn_blocking` + callback throttlé +
  annulation), ne bloque pas l'UI.
- **Index** : schéma correctement indexé pour égalité/tri/préfixe ; seule
  la recherche texte (`LIKE '%...%'`, pas de FTS5) fait un scan complet —
  point de dégradation potentiel si la bibliothèque grossit beaucoup.
- **Taille de `database.rs`** : due au nombre de responsabilités (10+
  tables), pas à des fonctions monstrueuses isolées ; les 5 plus grosses
  sont individuellement justifiées (migrations, DDL, pagination/tri/union
  Plex). Piste d'amélioration structurelle : fusionner les 3 variantes
  quasi-jumelles de requête d'albums et factoriser le pattern de migration
  répété — pas urgent, pas un bug.
- **`metadata.rs`** : complexité assumée et documentée (formats DSD, repli
  multi-tags pour bibliothèques mal taguées), pas de code accidentel.
