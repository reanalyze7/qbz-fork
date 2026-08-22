# crates/qbz-library/src/tag_sidecar.rs (165 lines)

## Summary
Album tag sidecar (`.qbz.json`) support for LocalLibrary: structs for
album/track metadata overrides, read/write/delete of the sidecar file, and
applying overrides onto a `LocalTrack`.

## Proposed split
Only ~35 lines over — split types/IO from the apply logic:

- `tag_sidecar/mod.rs` (~75 lines) — `AlbumMetadataOverride`,
  `TrackMetadataOverride`, `AlbumTagSidecar` + its `new()`, `SIDECAR_FILE_NAME`,
  `pub use` of `io` and `apply` submodules.
- `tag_sidecar/io.rs` (~40 lines) — `sidecar_path`, `read_album_sidecar`,
  `write_album_sidecar`, `delete_album_sidecar` (all filesystem I/O).
- `tag_sidecar/apply.rs` (~65 lines) — `apply_sidecar_to_track` + the private
  `normalize` helper (pure logic, no I/O).

## Re-export surface
`tag_sidecar/mod.rs` stays the `mod tag_sidecar;` target; all four public
fns (`sidecar_path`, `read_album_sidecar`, `write_album_sidecar`,
`delete_album_sidecar`, `apply_sidecar_to_track`) and three public types
re-exported via `pub use io::*; pub use apply::*;` so
`qbz_library::tag_sidecar::X` paths are unchanged.

## Coupling / watch out
- `write_album_sidecar` uses a tmp-file + rename pattern for atomicity —
  keep that in `io.rs` verbatim, don't "simplify" during the split.
- `apply_sidecar_to_track` takes `&mut LocalTrack` from `crate::LocalTrack`
  (defined elsewhere in qbz-library) — no change needed, just keep the
  `use crate::{LibraryError, LocalTrack};` import in `apply.rs`.
- No `#[cfg(test)]` module exists in this file today — flag that there's no
  regression safety net for the split; consider adding a couple of basic
  round-trip tests (write+read) as part of the real split PR.

## Verify after split
- `cargo build -p qbz-library` (no existing tests to run for this file).
- Manually exercise "edit album tags" (sidecar write) + reopening the album
  (sidecar read + apply) in the app, or add a temp-dir round-trip test.
