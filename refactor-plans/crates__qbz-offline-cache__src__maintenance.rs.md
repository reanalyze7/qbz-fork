# crates/qbz-offline-cache/src/maintenance.rs (144 lines)

## Summary
Pure maintenance operations on the offline cache — no Tauri/framework
state, callable from any future TUI or headless binary: bulk album removal
(DB rows + on-disk CMAF bundle dirs), a pre-flight cache-size-limit check
before queuing new downloads, and a filter for which cached tracks qualify
as re-download targets.

## Proposed split
Only 14 lines over budget — a light two-way split by responsibility is
enough:

- `maintenance/mod.rs` (~15 lines) — module doc, `pub use` re-exports of
  `AlbumRemovalReport`, `remove_album_cached_tracks`, `check_cache_limit`,
  `select_redownload_targets`.
- `maintenance/removal.rs` (~65 lines) — `AlbumRemovalReport` struct,
  `remove_album_cached_tracks()`, `check_cache_limit()` — the two
  operations that touch the DB/filesystem (deletion + limit check).
- `maintenance/redownload.rs` (~65 lines) — `select_redownload_targets()`
  (pure filter, no I/O) plus its existing `#[cfg(test)] mod tests`
  (`redownload_targets_full_skips_only_downloading`,
  `redownload_targets_failed_only_returns_failed`).

If the reviewer prefers not to introduce two new files for a 144-line file
barely over budget, an acceptable minimal alternative is a flat two-file
split without a directory: keep `maintenance.rs` (~90 lines: module doc,
`AlbumRemovalReport`, `remove_album_cached_tracks`, `check_cache_limit`)
and add a sibling `redownload.rs` (~55 lines: `select_redownload_targets`
+ its tests) with `pub use redownload::select_redownload_targets;` at the
top of `maintenance.rs`.

## Re-export surface
`maintenance/mod.rs` (or `maintenance.rs` in the flat alternative)
re-exports all four public items so
`crate::maintenance::{remove_album_cached_tracks, check_cache_limit,
select_redownload_targets, AlbumRemovalReport}` — used by the offline-cache
manager UI's "remove album" and "retry failed downloads" actions — stays
unchanged.

## Coupling / watch out
- `remove_album_cached_tracks` calls `db.delete_album_tracks(album_id)`
  (from `db.rs`, same batch) then, per returned track id, builds a
  `BundleLayout::new(offline_root, track_id)` (from `cmaf_store.rs`, same
  batch) and best-effort removes its directory — this function sits at the
  intersection of THREE files in this same review batch (`db.rs`,
  `cmaf_store.rs`, `maintenance.rs`); when all three get split for real,
  make sure `crate::db::OfflineCacheDb::delete_album_tracks` and
  `crate::cmaf_store::BundleLayout::new` paths still resolve from
  `maintenance/removal.rs`.
- Filesystem errors during per-track dir removal are explicitly
  logged-and-swallowed ("SQLite is the source of truth and the bundle
  directories are best-effort cleanup") — do not change this to propagate
  errors; the doc comment explaining why must travel with the code.
- `check_cache_limit` is intentionally simple per its own doc comment ("it
  does not predict the new track's size... sufficient for v1") — this is a
  known/accepted limitation, not a bug to fix during the split.
- `select_redownload_targets`'s status-filtering logic (`Downloading` always
  excluded; `Failed` always included; everything else gated by
  `failed_only`) is exactly what its two tests lock down — keep the tests
  colocated with the function so the behavior contract stays visible in
  one file.

## Verify after split
- `cargo test -p qbz-offline-cache maintenance` (or the new module path)
  — both `select_redownload_targets` tests must stay green.
- `cargo check -p qbz-offline-cache` / `cargo build -p qbz-offline-cache`
  to confirm cross-file calls into `db.rs` and `cmaf_store.rs` (both being
  split in this same batch) still resolve after all three land.
- Manually verify "remove album from offline cache" still deletes both the
  SQLite rows and the on-disk CMAF bundle directories, and that the
  cache-limit toast still appears when the configured limit is reached.
