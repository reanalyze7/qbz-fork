# crates/qbz-library/src/tag_writer.rs (169 lines)

## Summary
Frontend-agnostic embedded-tag writer (lofty-based) shared by the Slint and
Tauri frontends: `write_album_tags_to_files` (the main dedup + per-file
lofty write loop) plus a small pure helper `compute_track_artist_match`, and
the two data structs `AlbumTagWrite`/`TrackTagWrite`.

## Proposed split
By data-vs-I/O-vs-pure-logic:

- `tag_writer/mod.rs` (~15 lines) — module doc (lines 1-5) + `pub use
  types::{AlbumTagWrite, TrackTagWrite}; pub use write::write_album_tags_to_files;
  pub use match_artist::compute_track_artist_match;`.
- `tag_writer/types.rs` (~20 lines) — lines 11-28: `AlbumTagWrite`,
  `TrackTagWrite` struct definitions only.
- `tag_writer/write.rs` (~120 lines) — lines 30-146: `write_album_tags_to_files`
  itself (the dedup + lofty read/mutate/save loop). This is the file's one
  genuinely large function; if it needs to shrink further, extract the
  per-file tag-mutation block (lines 70-132, setting title/album/artist/
  track/disc/album-artist/year/genre/catalog-number on one `Tag`) into a
  private helper `fn apply_tag_fields(tag: &mut Tag, album: &AlbumTagWrite,
  track: &TrackTagWrite)` in the same file — that alone brings
  `write_album_tags_to_files`'s own body under 50 lines while
  `apply_tag_fields` is ~55, keeping the file as one cohesive ~120-line
  unit rather than forcing an arbitrary second file.
- `tag_writer/match_artist.rs` (~20 lines) — lines 148-168:
  `compute_track_artist_match` (pure, no lofty/IO dependency at all — only
  needs `LocalTrack`).

## Re-export surface
`tag_writer/mod.rs` re-exports `AlbumTagWrite`, `TrackTagWrite`,
`write_album_tags_to_files`, `compute_track_artist_match` so both frontends
(Slint `qbz` crate and the legacy Tauri command layer, if still present)
keep calling `qbz_library::tag_writer::write_album_tags_to_files(...)` and
`qbz_library::tag_writer::compute_track_artist_match(...)` unchanged.

## Coupling / watch out
- `write_album_tags_to_files` takes `&AlbumTagWrite` and `&[TrackTagWrite]`
  — `write.rs` needs `use super::types::{AlbumTagWrite, TrackTagWrite};`.
- `crate::{LibraryError, LocalTrack}` (the crate-root re-exports) are used
  by `write.rs` (`LibraryError`) and `match_artist.rs` (`LocalTrack`) —
  both need their own `use crate::{...}` line; don't assume a shared
  prelude.
- The function's doc comment (lines 30-34) explicitly notes this is
  "Partial-failure unsafe by design: returns `Err` on the first failing
  file with prior files already modified" — preserve this doc comment
  verbatim on `write_album_tags_to_files` wherever it lands; it documents
  a real, deliberate (not accidental) semantic that callers rely on.
- The `on_progress: impl FnMut(usize, usize)` callback parameter has no
  external dependency beyond being called with `(current, total)` before
  each file write — no coupling concern.
- No `#[cfg(test)]` module exists in this file — the lofty read/write path
  is presumably tested at a higher level (integration tests against real
  audio fixture files) elsewhere in the crate, if at all.

## Verify after split
- `cargo check -p qbz-library` and `cargo test -p qbz-library` (any
  existing integration tests touching `write_album_tags_to_files` or
  `compute_track_artist_match`).
- Smoke-test "Edit album tags" (direct/embedded write mode, not the
  sidecar-override mode) in the running app against a scratch copy of a
  real audio file, confirming title/album/artist/track/disc/genre/year/
  catalog-number all still write correctly and that removing a field
  (blank album_artist, `year: None`) still clears the corresponding tag —
  this is destructive-by-design so a real file round-trip is the only
  reliable check.
