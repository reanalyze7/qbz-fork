# `crates/qbz-library/src/database.rs` (6386 lines)

## 1. Summary
`LibraryDatabase`: the SQLite persistence layer for the local music library —
schema creation + ~20 sequential migrations, and CRUD/query methods for
folders, tracks, albums (incl. grouped/paginated metadata views), artists,
search, playlist settings/stats/folders/local-tracks/custom-order, album
settings, Qobuz-cached-track integration, artist images, custom covers,
offline-mode local-content detection, and downloaded-purchase tracking. Plus
~1000 lines of `#[cfg(test)]` tests. This is the single largest file in the
crate by a wide margin and needs a directory-per-domain split, not a single
extra module.

## 2. Proposed module layout

Convert `database.rs` into `database/` directory. Sizes are approximate line
counts derived from the existing `// === Section ===` markers already present
in the file (a good sign the domain boundaries are already implicit).

- `database/mod.rs` (~90 lines) — `LibraryDatabase` struct definition,
  `open()`, `with_connection`/`with_connection_mut`, module declarations,
  `pub use` re-exports of every submodule's public items so
  `qbz_library::database::LibraryDatabase` keeps its current API surface.
  **This is the re-export/public-API surface.**
- `database/kv.rs` (~30) — `get_kv`/`set_kv`.
- `database/schema/mod.rs` (~15) — orchestrates `init_schema()` +
  `run_migrations()` call order (called from `open()`).
- `database/schema/init.rs` (~180) — the `CREATE TABLE`/`CREATE INDEX` DDL
  batch (`init_schema`'s body). Still slightly over 130; split the DDL string
  into two `execute_batch` calls (core tables vs. playlist/artwork/download
  tables) across `schema/init_core.rs` and `schema/init_extra.rs` if the
  literal-string line count doesn't compress well as one function.
- `database/schema/migrations.rs` split by chronology into 3 files of ~180
  lines each (`migrations_v1.rs`, `migrations_v2.rs`, `migrations_v3.rs`),
  each holding a contiguous run of the existing sequential `has_X: bool =
  query_row(...)` migration blocks, called in order from `schema/mod.rs`.
  The sample_rate INTEGER->REAL table-rebuild migration (~140 lines alone,
  around old line 536-688) should get its own file
  (`schema/migrate_sample_rate_real.rs`, ~140, split header-only if needed).
- `database/folders.rs` split into:
  - `folders/crud.rs` (~110) — `add_folder`, `add_folder_with_network_info`,
    `remove_folder`, `get_folders`, `get_network_folder_paths`.
  - `folders/metadata.rs` (~110) — `get_folders_with_metadata`,
    `get_folder_by_id`, `update_folder_settings`, `set_folder_enabled`,
    `update_folder_scan_time`, `update_folder_path`, plus the
    `LibraryFolder` struct if not already defined elsewhere in the crate.
- `database/tracks/` :
  - `tracks/insert.rs` (~90) — `is_qobuz_cached_track_by_path`,
    `insert_track`.
  - `tracks/query.rs` (~60) — `get_track`, `get_track_by_path`,
    `get_all_track_paths`.
  - `tracks/delete.rs` (~100) — `delete_tracks_in_folder`,
    `delete_tracks_in_folder_prefixed`, `album_keys_in_folder`,
    `remove_folder_with_tracks`, `clear_all_tracks`, `delete_tracks_by_ids`.
- `database/albums/`:
  - `albums/filter.rs` (~40) — `get_albums`, `get_albums_with_filter` (thin
    wrappers delegating to...).
  - `albums/filter_sql.rs` (~130) — `get_albums_with_full_filter` (the large
    inline SQL — this one function alone is ~165 lines old; extract the two
    near-duplicate `include_hidden` branches of the SQL string into a shared
    `fn albums_query(include_hidden, source_filter, network_filter) ->
    String` helper to cut it under 130).
  - `albums/tracks.rs` (~40) — `get_album_tracks`.
  - `albums/metadata_grouped.rs` (~130) — `get_albums_metadata_grouped`.
  - `albums/metadata_page.rs` (~130) — `get_albums_metadata_page`,
    `get_albums_metadata_page_inner`.
  - `albums/metadata_count.rs` (~90) — `count_albums_metadata_for_page`.
  - `albums/metadata_tracks.rs` (~30) — `get_album_tracks_metadata`.
  - `albums/artwork.rs` (~110) — `get_albums_without_artwork`,
    `update_album_artwork`, `update_album_group_artwork`,
    `resolve_album_cover_fallback`.
  - `albums/group_metadata.rs` (~120) — `update_album_group_metadata`,
    `update_tracks_metadata_by_id`, `find_album_group_key`.
- `database/artists.rs` (~65) — `get_artists`, `get_artists_with_filter`.
- `database/search.rs` split into `search/query.rs` (~90, `search`,
  `search_with_filter`) and `search/paged.rs` (~80, `search_with_filter_page`).
- `database/stats.rs` (~50) — `count_all_local_tracks`, `get_stats`.
- `database/helpers/mod.rs` (~10) — re-exports.
- `database/helpers/row_mapping.rs` (~40) — `row_to_track`.
- `database/helpers/format_enum.rs` (~100) — `parse_format`, `AudioFormat`
  `Default`/`from_str`/`as_str` impls (whichever of these currently live in
  this file vs. `crate::AudioFormat` proper — verify at split time).
- `database/playlist_settings/` (was "Playlist Settings" section, ~412
  lines): `crud.rs` (~110: get/save), `sort_artwork.rs` (~100:
  update_playlist_sort/artwork/search_query), `visibility.rs` (~110:
  set_playlist_hidden/favorite, get_favorite_playlist_ids, mark/is copied),
  `position.rs` (~90: set_playlist_position, reorder_playlists).
- `database/playlist_stats.rs` (~100, fits as-is).
- `database/playlist_folders/` : `crud.rs` (~130: create/get_all/get/update),
  `membership.rs` (~135: delete, reorder, move_playlist_to_folder,
  get_playlists_in_folder — split further if over 130).
- `database/playlist_local_tracks/` : `crud.rs` (~135: add/remove/update
  position/clear — split add+remove into one file, update+clear into
  another if over 130), `query.rs` (~135: get_playlist_local_tracks,
  get_playlist_local_tracks_with_position, get_playlist_local_track_count,
  get_all_playlist_local_track_counts).
- `database/sidecar_position.rs` (~115, fits as-is) —
  `next_playlist_sidecar_position`, `heal_playlist_sidecar_positions`.
- `database/custom_order/` : `query_init.rs` (~90: get/init),
  `set_move.rs` (~135, split if needed: set_playlist_custom_order,
  move_playlist_track, has/clear_playlist_custom_order).
- `database/album_settings.rs` (~70, fits) — get/set hidden,
  get_hidden_albums.
- `database/qobuz_downloads/` : `query.rs` (~90: get_qobuz_download_tracks,
  track_exists_by_qobuz_id/path, repair_qobuz_cached_track_by_path),
  `insert.rs` (~140, split into `insert_direct.rs` +
  `insert_with_grouping.rs` if both large: insert_qobuz_cached_track_direct,
  insert_qobuz_cached_track_with_grouping, remove_qobuz_cached_track,
  remove_all_qobuz_cached_tracks).
- `database/artist_images.rs` split into `query.rs` (~80: get_artist_image,
  get_all_custom_artist_images, get_all_artist_image_urls,
  get_all_canonical_names) and `cache.rs` (~40: cache_artist_image,
  cache_artist_image_with_canonical).
- `database/custom_album_covers.rs` (~80, fits).
- `database/local_content/` (was "Offline Mode" section, ~215 lines):
  `detection.rs` (~100: has_local_track_by_qobuz_id/metadata,
  get_local_track_id_by_qobuz_id/metadata, get_tracks_with_local_copies),
  `playlist_status.rs` (~100: update_playlist_local_content_status,
  get_playlists_by_local_content).
- `database/purchases.rs` (~100, fits) — mark/remove downloaded purchase,
  get_downloaded_purchase_track_ids/formats.

### Tests
Co-locate each domain's tests with that domain module as
`#[cfg(test)] mod tests { ... }` at the bottom of the corresponding file
(idiomatic Rust, keeps the 130-line budget per file including tests where
small; where a domain's tests alone exceed ~100 lines, e.g. the metadata
grouping tests (~400 lines) and folder-tree tests (~460 lines), give them
sibling `tests.rs` files included via `#[cfg(test)] mod tests;` from the
domain module):
- `albums/metadata_grouped/tests.rs` (or split into 2: va-detection tests vs.
  folder-fallback tests) — the `fresh_db`/`insert_track_for_test`/
  `insert_full_track_for_test` helpers plus `metadata_group_*` tests
  (old lines ~5402-5799).
- `folder_tree/tests.rs`, split into `tests_children.rs` (list_folder_children
  + special-char tests) and `tests_recursive.rs` (recursive listing/counting
  + network-exclude tests) — old lines ~5800-6261.
- `sidecar_position/tests.rs` — old lines ~6262-6386.

## 3. Re-export / public API surface
`database/mod.rs` is the module other crates and the rest of `qbz-library`
import through today (`use qbz_library::database::LibraryDatabase` or
`use crate::database::{...}` internally). It must `pub use` every public
item from every submodule at the SAME paths they had as inherent methods on
`LibraryDatabase` — since all of these are `impl LibraryDatabase { pub fn
... }` blocks, the split is mechanical: each submodule gets its own
`impl LibraryDatabase { ... }` block for its slice of methods (Rust allows
multiple `impl` blocks for the same type across files/modules as long as
they're all declared as submodules of the crate). No re-export shimming is
actually needed for methods — only free functions/consts
(`AlbumTrackUpdate`, `TrackMetadataUpdateFull`, `LibraryFolder`, and any
other struct like `PlaylistSettings`/`PlaylistStats`/`PlaylistFolder` used
across methods) need `pub use` in `mod.rs` if they're moved out of the
top-level.

## 4. Tricky coupling to watch for
- All methods are `impl LibraryDatabase` — splitting across files is safe in
  Rust (inherent impls can be spread across many files) but every file needs
  `use super::LibraryDatabase;` (or `use crate::database::LibraryDatabase;`)
  and the same `use rusqlite::{params, Connection, OptionalExtension};` /
  `use crate::{AudioFormat, FolderTreeEntry, LibraryError, LocalAlbum,
  LocalArtist, LocalTrack};` imports currently declared once at file top.
- `schema::migrations` steps are STRICTLY ORDERED and several later
  migrations assume earlier ones already ran (e.g. the sample_rate rebuild
  re-adds `album_group_key`/`source`/`catalog_number` columns that may or
  may not exist depending on which earlier migrations already ran) — when
  splitting into `migrations_v1/v2/v3.rs`, preserve the exact call order in
  `schema/mod.rs::run_migrations()`, do not alphabetize or reorder.
  Cutting mid-migration or reordering will silently produce a different
  schema on upgrade paths.
  Also note `create_playlist_folder`/etc. and `run_migrations` both create
  `playlist_folders`/`playlist_settings.folder_id` — the migration and the
  "Playlist Folders" CRUD module both touch the same table; keep the
  `idx_playlist_settings_folder` index creation (currently done twice: once
  in migrations, once conditionally after) attached to migrations only.
- `TRACK_COLUMNS` const and `row_to_track` are used by many query modules
  (`tracks/query.rs`, `search/*.rs`, `albums/*`) — keep them in
  `helpers/row_mapping.rs` and import everywhere rather than duplicating.
- Local `AlbumTrackUpdate` / `TrackMetadataUpdateFull` structs declared at
  the top of the original file are used by `albums/group_metadata.rs` and
  possibly by callers outside `database.rs` (check `qbz-library/src/lib.rs`
  re-exports) — must stay `pub` and reachable at the same import path.
- Local playlists / Qobuz playlist snapshot: `open()` calls
  `crate::local_playlists::init_schema` and
  `crate::qobuz_playlist_snapshot::init_schema` — these live in SEPARATE
  files already (outside this split's scope) but are wired from
  `database/mod.rs::open()`; don't lose that wiring when restructuring
  `open()`.
- The `#[cfg(test)]` helper `fresh_db()` is redefined 3 times in the current
  file (once per test group) — when splitting tests into per-domain files,
  either de-duplicate into a shared `database/test_support.rs` (`#[cfg(test)]
  pub(crate) fn fresh_db(...)`) or accept the existing per-file duplication
  pattern (simpler, less coupling risk).

## 5. What to verify after the real split
- `cargo build -p qbz-library` and `cargo test -p qbz-library` — all
  existing tests (migration idempotency, metadata grouping, folder-tree
  primitives, sidecar-position healing) must stay green with identical
  behavior (these tests exercise real SQLite behavior, so any accidental
  reordering of migrations or SQL string mangling during extraction will
  surface here).
- `cargo build --workspace` to confirm no downstream crate (`qbz-app`,
  `qbzd`, `qbz-ui` bridge code) imported anything from `database.rs` by a
  path that no longer resolves (e.g. `qbz_library::database::LibraryFolder`).
- Grep the workspace for `qbz_library::database::` and
  `crate::database::` usages before/after to diff the accessible symbol set.
- Spot-check the schema migration order didn't change by diffing
  `run_migrations()`'s call sequence against the original file line order.
