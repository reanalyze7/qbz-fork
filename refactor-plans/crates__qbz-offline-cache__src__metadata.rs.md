# crates/qbz-offline-cache/src/metadata.rs (367 lines)

## Summary
Metadata fetching (from the Qobuz API) and FLAC tagging for offline-cached tracks:
builds `CompleteTrackMetadata` from track+album API responses, writes Vorbis
comment tags + embeds artwork into a FLAC file via `lofty`, sanitizes filenames, and
organizes a downloaded temp file into the final `<artist>/<album>/NN - Title.flac`
folder structure.

## Proposed split
By responsibility (fetch/IO vs pure filename logic vs FLAC-tag IO vs file-org IO) —
follows the pure/IO split fairly directly.

- `metadata/mod.rs` (~15 lines) — module doc + `pub use` re-exports of
  `CompleteTrackMetadata`, `fetch_complete_metadata`, `write_flac_tags`,
  `embed_artwork`, `sanitize_filename`, `save_album_artwork`, `organize_cached_file`
  so `crate::metadata::X` paths are unchanged.
- `metadata/model.rs` (~30 lines) — `CompleteTrackMetadata` struct definition only.
- `metadata/fetch.rs` (~80 lines) — `fetch_complete_metadata` (Qobuz API calls +
  field mapping/derivation: album_artist, genre, label, year, artwork_url).
- `metadata/tags.rs` (~75 lines) — `write_flac_tags` (lofty tag writing).
- `metadata/artwork.rs` (~55 lines) — `embed_artwork` (download + embed cover into
  the FLAC's primary tag) and `save_album_artwork` (download cover.jpg alongside the
  album folder) — both are "fetch artwork bytes and write them somewhere" so they
  pair naturally; split into two files if either grows (currently both fit under 130
  combined).
- `metadata/filename.rs` (~40 lines) — `sanitize_filename` (pure string sanitizing,
  no I/O — genuinely the "pure" module here).
- `metadata/organize.rs` (~70 lines) — `organize_cached_file` (path building + rename
  I/O), depends on `sanitize_filename` from `filename.rs`.

## Re-export surface
`metadata/mod.rs` re-exports every public item so all existing
`qbz_offline_cache::metadata::{fetch_complete_metadata, write_flac_tags,
embed_artwork, sanitize_filename, save_album_artwork, organize_cached_file,
CompleteTrackMetadata}` call sites (used by the cache-download orchestrator
elsewhere in this crate, and possibly by `qbzd`) compile unchanged.

## Coupling / watch out
- `organize_cached_file` calls `sanitize_filename` twice (artist_dir, album_dir, and
  again for `title_clean`) — straightforward cross-module `use super::filename::sanitize_filename;`.
  `sanitize_filename` also gets called a second time inside its own conflict-avoidance
  loop within `organize_cached_file` — no change needed, just confirm the import.
- `fetch_complete_metadata` takes `&qbz_qobuz::QobuzClient` — this is the crate's only
  external dependency touch point in this file; keep the import in `fetch.rs` only.
- `write_flac_tags` and `embed_artwork` both open/save the same file via
  `lofty::read_from_path`/`save_to_path` — they are called sequentially by the
  orchestrator (tag write, then artwork embed) as two separate read-modify-write
  passes on disk; this is pre-existing behavior (not a bug to fix here), just note it
  when splitting so no one "optimizes" it into a single open during the split (that
  would be a behavior change, out of scope for a pure file-split PR).
- All functions return `Result<_, String>` (no shared custom error type) — no shared
  state/type to worry about, low coupling risk overall.

## Verify after split
- `cargo test -p qbz-offline-cache metadata` (check if any unit tests exist for
  `sanitize_filename`/`organize_cached_file`; if none exist yet, that's pre-existing
  and out of scope, but flag it since the project rule wants tests on every change —
  consider adding a couple of `sanitize_filename` unit tests as this file gets split,
  since it's pure and trivially testable).
- `cargo check -p qbz-offline-cache` for the cache-download pipeline call site that
  chains `fetch_complete_metadata` → `write_flac_tags` → `embed_artwork` →
  `organize_cached_file`.
- Manual smoke-test: cache a track offline and confirm the resulting FLAC file has
  correct tags/artwork and lands in the expected folder path.
