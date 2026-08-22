# crates/qbz-offline-cache/src/migration.rs (261 lines)

## Summary
Legacy cached-file migration service: detects old numeric-named FLAC files
under the tracks/ folder, and migrates each one (fetch metadata, write tags,
embed/save artwork, reorganize into the artist/album folder structure, read
audio properties, insert into the local library DB) with a progress/error
status struct.

## Proposed split
Split by "detect" vs. "single-track migration" vs. "batch driver", which
matches the file's existing three top-level functions plus its two DTOs:

- `migration/mod.rs` (~35 lines) — module doc, `MigrationStatus` +
  `MigrationError` structs (`#[derive(Serialize, Clone, Debug)]`), `pub use`
  of the functions below.
- `migration/detect.rs` (~40 lines) — `detect_legacy_cached_files` (the
  directory scan for numeric-named `.flac` files) — a self-contained,
  synchronous, easily-tested function.
- `migration/single_track.rs` (~100 lines) — `migrate_single_track` (private
  async fn, lines 77-171): fetch metadata → write tags → embed artwork →
  organize file → save album artwork → read audio properties (lofty) →
  insert into library DB. This is the biggest single function and the core
  "how to migrate one track" logic; keep it as one function (it's already a
  clear numbered 1-7 step sequence per its own comments) rather than
  splitting further, since each step depends on the previous step's output
  (`new_path`, `metadata`) threading through sequentially.
- `migration/batch.rs` (~90 lines) — `migrate_legacy_cached_files` (the
  public async batch driver, lines 174-261): iterates track_ids, locks the
  `QobuzClient`, calls `migrate_single_track`, updates `MigrationStatus`,
  deletes the legacy file on success, logs summary.

## Re-export surface
`migration/mod.rs` re-exports `MigrationStatus`, `MigrationError`,
`detect_legacy_cached_files`, `migrate_legacy_cached_files` at
`crate::migration::*` (or wherever `qbz_offline_cache::migration::` is
re-exported from the crate root today — check `qbz-offline-cache/src/lib.rs`'s
existing `pub mod migration;` and keep it unchanged). `migrate_single_track`
stays private (`fn`, not `pub`) exactly as today, only reachable from
`batch.rs` via `use super::single_track::migrate_single_track;`.

## Coupling / watch out
- `migrate_single_track` (single_track.rs) is called ONLY by
  `migrate_legacy_cached_files` (batch.rs) — make it `pub(super)` (or
  `pub(crate)`) rather than a bare private `fn`, since it now crosses a file
  boundary within the same module even though it stays invisible outside
  `migration/`.
- Both `single_track.rs` and `batch.rs` import from
  `crate::metadata::{embed_artwork, fetch_complete_metadata,
  organize_cached_file, save_album_artwork, write_flac_tags}` — but only
  `single_track.rs` actually calls them; `batch.rs` doesn't need that `use`
  line, only `qbz_library::LibraryDatabase` and `qbz_qobuz::QobuzClient` (for
  the `Arc<Mutex<...>>`/`Arc<RwLock<...>>` parameter types) plus `super::single_track::migrate_single_track`
  and `super::{MigrationStatus, MigrationError}`.
- `library_db: Arc<Mutex<Option<LibraryDatabase>>>` is threaded through from
  `migrate_legacy_cached_files` (batch.rs) into `migrate_single_track`
  (single_track.rs) as a plain parameter (cloned `Arc`, not shared mutable
  struct state) — no special handling beyond making sure both files' function
  signatures still match exactly; this is NOT a struct field, so there's no
  "keep the field in one place" concern here, just a parameter-passing
  contract between the two files.
- `detect_legacy_cached_files` is synchronous and has zero dependency on the
  other two functions or their imports (`lofty`, `qbz_library`, `qbz_qobuz`)
  — confirms it's a clean, independent module with only `std::fs`/`std::path`
  needs.
- No `#[cfg(test)]` block exists in this file today — nothing to extract on
  that front, but note for the actual implementer that this file currently
  has ZERO unit tests, which is a gap the split should not worsen (consider
  flagging to the owner that `detect_legacy_cached_files` in particular is
  easily unit-testable with a tempdir and should probably gain tests as part
  of or after this split, per the project's "tests at each change" rule).

## Verify after split
- `cargo check -p qbz-offline-cache` (no existing tests to run for this file
  specifically) and grep for `migration::migrate_legacy_cached_files` /
  `migration::detect_legacy_cached_files` / `MigrationStatus` importers
  (likely a Tauri/Slint settings command that triggers the one-time migration
  on startup or on user request) to confirm the public path and struct shape
  are unchanged.
- Given the "tests at each change" project rule and this file having none
  today, the actual split PR should add at minimum a
  `detect_legacy_cached_files` unit test (tempdir with a few numeric `.flac`
  files + a non-numeric one + a non-flac one) before/alongside the split.
