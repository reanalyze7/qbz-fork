# crates/qbz-app/src/settings/favorites_cache.rs (455 lines)

## Summary
Frontend-agnostic local SQLite cache (`favorites_cache.db`) of favorite
track/album/artist/label IDs, hoisted verbatim from the Tauri store so
non-Tauri frontends can read favorite status offline; one repetitive CRUD
block per entity kind plus a `clear_all` and a `#[cfg(test)]` module.

## Proposed split
The four entity blocks are near-identical CRUD (get_ids/is_favorite/add/
remove/sync) — split by entity, one file per kind, plus the shared
open/schema code and the tests:

- `favorites_cache/mod.rs` (~60 lines) — module doc, `FavoritesCacheStore`
  struct definition + `open_at` (schema creation for all 4 tables — keep
  the CREATE TABLE statements together since they run in one `open_at` call),
  `new`, `new_at`, and `clear_all` (touches all 4 tables, belongs with the
  struct itself). Re-exports nothing extra — the impl blocks in the other
  files all attach to the same `FavoritesCacheStore` type via `impl
  FavoritesCacheStore` blocks (Rust allows splitting impls of one struct
  across files/modules as long as they're all part of the same crate — no
  re-export gymnastics needed, just multiple `impl` blocks).
- `favorites_cache/tracks.rs` (~70 lines) — lines 87-152: the
  `impl FavoritesCacheStore` block with `get_favorite_track_ids`,
  `is_track_favorite`, `add_favorite_track`, `remove_favorite_track`,
  `sync_favorite_tracks`.
- `favorites_cache/albums.rs` (~70 lines) — lines 156-220: same shape for
  albums (note: `album_id` is `&str`/`String`, not `i64`, unlike the other
  three).
- `favorites_cache/artists.rs` (~70 lines) — lines 224-288: same shape for
  artists.
- `favorites_cache/labels.rs` (~70 lines) — lines 292-356: same shape for
  labels.
- `favorites_cache/tests.rs` (~80 lines) — lines 377-455: the `#[cfg(test)]
  mod tests` block, `use super::super::FavoritesCacheStore` (or re-export
  path), unchanged.

## Re-export surface
`favorites_cache/mod.rs` declares `pub struct FavoritesCacheStore` and
`mod tracks; mod albums; mod artists; mod labels; #[cfg(test)] mod tests;` —
since each of those files only adds `impl` blocks on the already-`pub`
struct (no new pub items to re-export), `crate::settings::favorites_cache::
FavoritesCacheStore` stays the exact same import path with the exact same
method set. No caller-visible change at all.

## Coupling / watch out
- All 4 entity impl blocks + `open_at`/`clear_all` operate on the single
  `self.conn: Connection` field — no shared mutable state beyond that, so
  the split is low-risk (this is the friendliest file in this batch).
- `open_at`'s CREATE TABLE statements must all stay in ONE function (in
  mod.rs) since they run once at construction — do not try to move a
  table's schema into its entity file, that would just add indirection with
  no benefit and risks re-ordering the creation calls.
- The db filename (`"favorites_cache.db"`) and schema must stay byte-
  identical to the Tauri original per the file's own doc comment — the
  split must not touch column names/types, only file boundaries.

## Verify after split
- `cargo test -p qbz-app favorites_cache` (existing 4 tests must stay green:
  `favorites_cache_track_ids_roundtrip`,
  `favorites_cache_add_track_is_idempotent`,
  `favorites_cache_sync_replaces_existing_track_set`,
  `favorites_cache_other_entities_roundtrip`).
- `cargo check -p qbz-app` / `cargo build -p qbz-app`.
- Smoke-test importers: search `crate::settings::favorites_cache::
  FavoritesCacheStore` usages (likely in `qbz`'s `fav_cache.rs` or similar)
  compile unchanged.
