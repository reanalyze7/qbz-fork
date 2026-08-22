# crates/qbz-audio/src/loudness.rs (376 lines)

## Summary
ReplayGain extraction (from Symphonia-probed audio, both file bytes and an
already-open `FormatReader`) and gain-factor calculation (dB -> linear
amplitude, with clipping prevention) for loudness normalization.

## Proposed split
By pure/IO-adjacent boundary — extraction (touches Symphonia's
probe/metadata "IO-shaped" API) vs. calculation (pure math) vs. the small
`MediaSource` adapter shim:

- `loudness/mod.rs` (~20 lines) — module doc, `ReplayGainData` struct,
  `mod` wiring (`mod source_adapter; mod extract; mod gain;`), re-exports.
- `loudness/source_adapter.rs` (~35 lines) — `CursorMediaSource` struct +
  its `Read`/`Seek`/`MediaSource` impls (a small, self-contained shim with no
  dependency on the rest of the file).
- `loudness/extract.rs` (~125 lines) — `extract_replaygain`,
  `extract_replaygain_from_reader`, `extract_from_tags` (the Symphonia-facing
  extraction logic; keep these three together since `extract_from_tags` is
  the shared tag-scanning helper both public fns call).
- `loudness/gain.rs` (~110 lines) — `parse_gain_value`, `parse_peak_value`,
  `value_to_string`, `db_to_linear`, `calculate_gain_factor` (pure
  computation, no Symphonia I/O types beyond the `Value` enum for parsing).
- `loudness/tests.rs` (~95 lines) — the `#[cfg(test)] mod tests` block,
  wired via `#[cfg(test)] mod tests;` in `mod.rs`.

## Re-export surface
`loudness/mod.rs` — becomes `crates/qbz-audio/src/loudness/mod.rs`. Keeps
`pub struct ReplayGainData`, `pub fn extract_replaygain`,
`pub fn extract_replaygain_from_reader`, `pub fn db_to_linear`,
`pub fn calculate_gain_factor` all re-exported (via `pub use extract::*;
pub use gain::{db_to_linear, calculate_gain_factor};` or by leaving the
`pub fn`s directly accessible through `mod` visibility — either way,
`qbz_audio::loudness::extract_replaygain` etc. must resolve unchanged).
`CursorMediaSource` and `extract_from_tags`/`parse_gain_value`/
`parse_peak_value`/`value_to_string` stay private (`pub(crate)` at most, if
needed by tests in a different file — but tests already live in the same
module tree so plain module-private is fine).

## Coupling / watch out
- `extract_from_tags` is called by BOTH `extract_replaygain` and
  `extract_replaygain_from_reader` — keep it in `extract.rs` alongside both
  callers rather than moving it to `gain.rs`, even though it's arguably
  "parsing" — it operates on Symphonia `Tag`/`Value` types tied to the
  extraction path, not the dB-math path.
- `parse_gain_value`/`parse_peak_value`/`value_to_string` are used only by
  `extract_from_tags` (in `extract.rs`) but are proposed for `gain.rs` — this
  creates an `extract.rs -> gain.rs` dependency. Alternative: keep these three
  parse helpers in `extract.rs` next to their only caller, and let `gain.rs`
  hold ONLY `db_to_linear`/`calculate_gain_factor` (true pure math, zero
  Symphonia types). Pick whichever grouping a later editor prefers — noting
  it here since the "pure vs IO" boundary is genuinely ambiguous for these
  three parse helpers (they're pure functions but operate on IO-crate types).
- `CursorMediaSource` has no test coverage of its own and no other file
  dependency — safe, isolated extraction.
- Test module tests `db_to_linear`, `calculate_gain_factor`, and
  `parse_gain_value`/`parse_peak_value` directly — after the split, `tests.rs`
  needs `use super::gain::*;` (or `super::extract::*` depending on where the
  parse helpers land) plus `use super::ReplayGainData;` from `mod.rs`.

## Verify after split
- `cargo test -p qbz-audio loudness` — all 7 existing tests green.
- `cargo check -p qbz-audio` for the crate itself, plus any downstream crate
  using `qbz_audio::loudness::{extract_replaygain, calculate_gain_factor,
  db_to_linear, ReplayGainData}` (likely the playback/normalization pipeline).
