# crates/qbz-library/src/cue_parser.rs (314 lines)

## Summary
CUE-sheet parser for single-file albums: parses `.cue` text into a
`CueSheet`/`CueTrack` model (`CueParser::parse`), and converts a parsed sheet
plus audio duration/format/properties into a `Vec<LocalTrack>`
(`cue_to_tracks`).

## Proposed split
Split by responsibility: data model, the line-by-line parser, the
LocalTrack-conversion, and tests.

- `cue_parser/mod.rs` (~25 lines) — module doc, `pub mod` decls, `pub use`
  re-exports of `CueSheet`, `CueTrack`, `CueTime`, `CueParser`,
  `cue_to_tracks`.
- `cue_parser/model.rs` (~60 lines) — `CueSheet`, `CueTrack`, `CueTime`
  structs + `CueTime::to_seconds`/`CueTime::parse`.
- `cue_parser/parse.rs` (~130 lines) — `CueParser` struct + `impl` block:
  `parse` (file read with UTF-8/Latin-1 fallback) + `parse_content` (the
  line-by-line state machine) — this is already right at the 130-line edge
  on its own; if it runs over, split `extract_quoted`/`extract_track_number`
  into a small `cue_parser/parse_helpers.rs` (~20 lines) to bring
  `parse.rs` under the limit.
- `cue_parser/convert.rs` (~70 lines) — the free fn `cue_to_tracks`
  (CueSheet -> `Vec<LocalTrack>`).
- `cue_parser/tests.rs` (~35 lines) — the `#[cfg(test)] mod tests` block
  (`test_cue_time_parse`, `test_extract_quoted`, `test_extract_track_number`).

## Re-export surface
`cue_parser/mod.rs` re-exports `CueSheet`, `CueTrack`, `CueTime`, `CueParser`,
`cue_to_tracks` at `crate::cue_parser::*` (i.e. `qbz_library::cue_parser::*`)
— the library-scanning code elsewhere in `qbz-library` that detects `.cue`
sidecar files and the `qbz-app`/`qbz` crates that surface local-library
tracks depend on this exact path.

## Coupling / watch out
- `CueParser::extract_quoted`/`extract_track_number` are currently private
  associated fns (`impl CueParser`), directly unit-tested via
  `CueParser::extract_quoted(...)` — if pulled into a separate
  `parse_helpers.rs` file, decide whether they stay as `CueParser::` assoc
  fns (impl block split across files, fine in Rust) or become free fns; the
  existing tests call them as `CueParser::extract_quoted(...)`, so keep them
  as associated functions on `CueParser` to avoid touching the tests.
- `cue_to_tracks` depends on `MetadataExtractor::album_group_info` and
  `MetadataExtractor::infer_disc_number` (from `crate::{AudioFormat,
  AudioProperties, LibraryError, LocalTrack, MetadataExtractor}` at the top
  of the file) — these are external crate items, not affected by the split,
  just keep the `use` statement in whichever file needs them (`convert.rs`).
- `LibraryError::CueParse` variant is constructed in `parse_content` for two
  validation failures (missing FILE directive, zero tracks) — no change
  needed, just verify the `use` path resolves in `parse.rs`.

## Verify after split
- `cargo test -p qbz-library cue_parser::` — all 3 existing tests green.
- `cargo check -p qbz-library` and grep for `cue_parser::`/`CueParser`/
  `cue_to_tracks` usages in the local-library scanner to confirm the public
  path is unchanged.
- Manual/smoke: scan a local-library folder containing a `.cue`+audio pair
  and confirm tracks still appear correctly split (title/performer/start
  times) in the LocalLibrary view.
