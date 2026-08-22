# crates/qbz-library/src/ephemeral.rs (435 lines)

## Summary
In-memory "open an ad-hoc folder and play it" library: synthetic high-id
track allocation, a mutex-guarded `EphemeralLibraryInner`/
`EphemeralLibraryState`, and the big `open_folder` scan (CUE-sheet
expansion + plain audio-file extraction, both with per-album artwork
caching) that populates it.

## Proposed split
By responsibility — types/state shell vs the two scan sub-passes (CUE vs
plain audio) vs the final sort/re-key step — as a `ephemeral/` directory
module:

- `ephemeral/mod.rs` (~90 lines) — module doc (lines 1-29), `pub use`
  re-exports, `EPHEMERAL_ID_FLOOR` const, `EphemeralFolderResult`,
  `EphemeralError` + its `Display`/`From<LibraryError>` impls (lines 43-73),
  and the `EphemeralLibraryState`/`EphemeralLibraryInner` struct
  definitions + `new`/`reset`/`Default` (lines 75-106, 431-435). This is
  the module's public shape and shared state.
- `ephemeral/scan_cue.rs` (~120 lines) — extract the CUE-file loop (lines
  156-266) as a free function `scan_cue_files(scan, inner, artwork_cache,
  album_artwork_cache) -> (Vec<LocalTrack>, usize skipped, HashSet<PathBuf>
  cue_referenced_audio)` (or take `&mut` refs to the accumulators) so
  `open_folder` becomes a thin orchestrator.
- `ephemeral/scan_audio.rs` (~110 lines) — extract the plain-audio-file
  loop (lines 268-355) similarly, taking `cue_referenced_audio` to skip
  already-claimed files.
- `ephemeral/open_folder.rs` (~90 lines) — the `open_folder` method itself
  (lines 111-397): validates the path, calls `LibraryScanner`, sets up the
  two artwork caches, calls into `scan_cue.rs`/`scan_audio.rs`, does the
  final musical-order sort + id re-key (lines 362-380), and logs/returns
  the result. This is `impl EphemeralLibraryState` — kept as a method via
  `impl` block continuation (Rust allows multiple `impl` blocks for the
  same type across files in one crate/module).
- `ephemeral/query.rs` (~40 lines) — `clear`, `get_track`,
  `tracks_snapshot`, `current_folder_path` (lines 399-428) — the small
  read/clear accessor methods, as another `impl EphemeralLibraryState`
  block.

## Re-export surface
`ephemeral/mod.rs` stays the public surface: `pub use` (or the types are
simply defined there) so `qbz_library::ephemeral::{EphemeralLibraryState,
EphemeralFolderResult, EphemeralError, EPHEMERAL_ID_FLOOR}` resolves
unchanged. The crate's `lib.rs` line `pub mod ephemeral;` needs no change.
Per the file's own doc comment, this module is re-exported downstream by
`src-tauri/src/ephemeral_library/mod.rs` (Tauri) and wrapped in a
process-global singleton by `crates/qbz-slint/src/ephemeral.rs` (Slint) —
both are OUTSIDE this crate and depend only on the same public names, so
they are unaffected as long as `ephemeral::EphemeralLibraryState`'s public
method signatures (`open_folder`, `clear`, `get_track`, `tracks_snapshot`,
`current_folder_path`) don't change.

## Coupling / watch out
- `EphemeralLibraryInner` fields (`tracks`, `next_id`,
  `current_folder_path`) are private to the struct and mutated across the
  CUE loop, audio loop, and the final re-key step — since `open_folder`
  takes `let mut inner = self.inner.lock()...` ONCE and threads `&mut
  inner` through both scan passes, the split functions in `scan_cue.rs`/
  `scan_audio.rs` must take `&mut EphemeralLibraryInner` (or the specific
  fields) as a parameter rather than re-locking — re-locking inside a
  sub-function would deadlock (the outer `open_folder` already holds the
  lock for the whole scan).
- `album_artwork_cache`/`folder_artwork_cache`/`cue_referenced_audio` are
  local accumulators in the current single function, threaded by mutable
  reference across both loops (the audio loop checks
  `cue_referenced_audio.contains` to skip CUE-claimed files) — when split,
  these must be created in `open_folder.rs` and passed as `&mut` into both
  `scan_cue.rs` and `scan_audio.rs`, in that order (CUE first, so its
  claimed-files set exists before the audio pass runs).
- The final sort-then-re-key step (lines 362-380) depends on BOTH scan
  passes having already populated `tracks_out`/`inner.tracks` — it must run
  in `open_folder.rs` after both sub-scans return, not be hoisted into
  either sub-module.
- `crate::get_artwork_cache_dir()` and the CUE/metadata extraction helpers
  (`cue_to_tracks`, `CueParser`, `LibraryScanner`, `LocalTrack`,
  `MetadataExtractor`) are all `crate::`-level imports already at the top
  of the file — each new sub-file needs its own `use crate::{...}` (or
  `use super::*` where the mod.rs re-exports them).

## Verify after split
- `cargo build -p qbz-library`.
- `cargo test -p qbz-library ephemeral::` (check for existing tests in this
  module or its callers — this file itself has none inline; verify
  integration tests elsewhere in the crate/workspace that exercise
  `open_folder` still pass).
- `cargo clippy -p qbz-library`.
- Smoke-test importers: `grep -rn "ephemeral::" crates/qbz-slint crates
  src-tauri` (if present) — confirm the Tauri re-export and the Slint
  singleton wrapper still compile against
  `EphemeralLibraryState::{new, open_folder, clear, get_track,
  tracks_snapshot, current_folder_path}` and `EphemeralFolderResult`/
  `EphemeralError`.
- Manually smoke-test: open an ephemeral folder containing both plain
  audio files and a CUE+FLAC pair, verify tracks appear in musical order
  with sequential synthetic ids, verify APE files are skipped, verify
  re-opening a different folder clears the previous session.
