# crates/qbz-offline-cache/src/db.rs (753 lines)

## Summary
`OfflineCacheDb` — the SQLite wrapper around the `cached_tracks` index
table: schema init + additive v2-CMAF migration, CRUD for cache entries
(insert/update-status/update-progress/mark-complete/touch/delete), LRU
eviction queries, aggregate stats, and the v2 CMAF-bundle-specific column
read/write pair (`set_cmaf_bundle`/`get_cmaf_bundle`).

## Proposed split
By domain — schema/migration vs. legacy-track CRUD vs. CMAF-bundle fields
vs. album-scoped bulk ops — mirroring the file's own `// ====` banner
comments:

- `db/mod.rs` (~20 lines) — module doc, `pub use` of `OfflineCacheDb`,
  `CmafBundleRow`; keeps `row_to_cached_track_info` as a private helper
  imported by whichever files run the 17-column SELECT.
- `db/schema.rs` (~100 lines) — `OfflineCacheDb::new`, `conn()`,
  `init_schema`, `migrate_v2_cmaf_columns`, `existing_columns` — table
  creation + the additive migration, and the `struct OfflineCacheDb { conn }`
  definition itself.
- `db/tracks.rs` (~230 lines) — the core per-track CRUD: `insert_track`,
  `insert_tracks_batch`, `update_status`, `update_progress`, `mark_complete`,
  `touch`, `is_cached`, `get_file_path`, `get_track`, `get_all_tracks`,
  `update_file_path`, `update_artwork_path`, plus the shared
  `row_to_cached_track_info` mapper function.
- `db/sync.rs` (~35 lines) — `get_ready_tracks_for_sync` (a distinct
  query shape/purpose: syncing ready tracks into the library).
- `db/album.rs` (~90 lines) — `get_album_tracks`,
  `reset_track_for_redownload`, `delete_album_tracks` — the album-scoped
  and re-download-reset operations, which is also exactly what the existing
  `maintenance_tests` module exercises.
- `db/stats.rs` (~110 lines) — `get_stats`, `get_tracks_for_eviction`,
  `clear_all` — aggregate/whole-table operations for the cache-manager UI.
- `db/cmaf.rs` (~70 lines) — `set_cmaf_bundle`, `get_cmaf_bundle`, and the
  `CmafBundleRow` struct + its doc comment explaining `cache_format` 1 vs 2.
- `db/tests.rs` (~75 lines) — the existing `#[cfg(test)] mod
  maintenance_tests`, moved as-is (it exercises `delete_album_tracks`,
  `get_album_tracks`, `reset_track_for_redownload` — all in `db/album.rs`,
  so co-locate the test module there instead of a separate file, i.e. put
  it at the bottom of `db/album.rs`).

Since `OfflineCacheDb` is one struct with `impl` blocks spread across all
these files, Rust allows multiple `impl OfflineCacheDb { ... }` blocks
across files in the same module — no trait needed, just repeat
`impl OfflineCacheDb` in each file with its subset of methods.

## Re-export surface
`db/mod.rs` re-exports `pub use schema::OfflineCacheDb;` (the struct is
defined once in `schema.rs`) and `pub use cmaf::CmafBundleRow;`. All other
files (`tracks.rs`, `sync.rs`, `album.rs`, `stats.rs`, `cmaf.rs`) add
`impl OfflineCacheDb` blocks via `use super::schema::OfflineCacheDb;` — they
don't need their own re-export since they're extending the type, not
defining new public symbols (except `CmafBundleRow`). Every caller today
uses `crate::db::OfflineCacheDb` or the crate-root re-export — unaffected.

## Coupling / watch out
- `row_to_cached_track_info` (currently a free fn, not a method) is called
  from THREE places: `get_track`, `get_all_tracks` (both `tracks.rs`), and
  `get_album_tracks` (`album.rs`) — make it `pub(super)` or `pub(crate)` in
  `tracks.rs` and `use super::tracks::row_to_cached_track_info;` from
  `album.rs`, OR hoist it to `mod.rs` as a shared crate-visible helper
  since it's used across the split. The doc comment above it enumerating
  the exact 17-column SELECT order is CRITICAL — the column order must
  match across all three callers; keep the comment attached and do not
  let any caller's SELECT drift from that exact ordering when relocating.
- `existing_columns` + `migrate_v2_cmaf_columns` in `schema.rs` must run
  before any v2 CMAF column is referenced — `new()` calls `init_schema()`
  which calls `migrate_v2_cmaf_columns()` at the end — keep this call
  order intact.
- `conn()` (pub, returns `&Connection`) is used directly by
  `get_album_tracks` via `self.conn()` instead of `self.conn` field access
  — both patterns exist in the file (`self.conn` and `self.conn()`); after
  the split, all files still call through `self.conn()` or the private
  field since they're all `impl OfflineCacheDb` blocks with access to the
  private `conn` field (private-to-module, so fine as long as everything
  stays under `db/`).
- `delete_album_tracks` uses `self.conn().unchecked_transaction()` for
  atomicity across the SELECT-ids + SELECT-sum-bytes + DELETE — keep the
  transaction boundary intact when moving to `album.rs`.
- `set_cmaf_bundle`/`get_cmaf_bundle` reference `cache_format`, `init_path`,
  `content_key_wrapped`, `infos_wrapped`, `format_id`, `n_segments` —
  exactly the columns `migrate_v2_cmaf_columns` (in `schema.rs`) adds; the
  two files are coupled by column names only (no code coupling), but a
  future column rename must touch both.

## Verify after split
- `cargo test -p qbz-offline-cache` — the `maintenance_tests` module
  (3 tests: `delete_album_tracks_returns_deleted_ids_and_freed_bytes`,
  `get_album_tracks_returns_only_matching_album`,
  `reset_track_for_redownload_clears_progress_and_error`) must stay green.
- `cargo check -p qbz-offline-cache` and `cargo build -p qbz-offline-cache`
  to confirm `downloader.rs`, `maintenance.rs`, `playback.rs` (all callers
  of `OfflineCacheDb` methods, in this same batch) still compile against
  the split API.
- Manually verify a fresh install still creates the schema correctly and
  an existing v1 install still migrates (adds the 6 new columns) without
  data loss — this is genuinely risky code (schema migration) so a
  smoke-test against a real pre-migration DB file is worth doing before/
  after.
