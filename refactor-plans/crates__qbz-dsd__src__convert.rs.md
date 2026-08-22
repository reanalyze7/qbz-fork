# `crates/qbz-dsd/src/convert.rs` (283 lines)

## 1. Summary
Streaming DSD → 88.2 kHz PCM conversion chain: dsd2pcm decimation + a
chain of half-band ÷2 FIR stages down to a uniform output rate, wrapped in
a `DsdPcmConverter` that also downmixes multichannel DSD to stereo, plus
unit tests for the half-band filter's DC gain and output-count invariants.

## 2. Proposed module layout

Convert to `convert/` directory:

- `convert/mod.rs` (~20) — module doc comment, `mod` declarations, `pub use`
  re-exports of `OUTPUT_RATE`, `DEFAULT_GAIN_DB`, `DsdPcmConverter`. **This
  is the re-export/public-API surface.**
- `convert/halfband.rs` (~110) — `HALFBAND_TAPS` const, `halfband_taps()`
  (windowed-sinc coefficient generator), `HalfBand` struct + its `new()`/
  `process()` impl. This is a fully self-contained, independently testable
  unit already.
- `convert/downmix.rs` (~55) — extract the ITU-R BS.775 stereo-downmix math
  currently inline inside `next_block()` (the `K`/`ci`/`sli`/`sri`/`norm`
  computation and the per-frame `match self.channels { ... }` fold) into a
  free function, e.g. `fn fold_to_stereo(channels: usize, per_ch: &[Vec<f32>],
  frame: usize) -> (f32, f32)`, or a small struct precomputing
  `(ci, sli, sri, norm)` once per converter instead of implicitly once per
  `next_block` call (a minor efficiency note, not required, but natural
  once it's its own module). This turns `next_block` into a much shorter
  caller.
- `convert/converter.rs` (~110) — `DsdPcmConverter` struct definition,
  `new()`, `output_rate()`/`channels()`/`total_frames()`, and `next_block()`
  (now calling `downmix::fold_to_stereo` instead of inlining the math).
- `convert/tests.rs` (~35) — the existing `#[cfg(test)] mod tests` (both
  half-band tests). Small enough to leave as one file; could equally stay
  as `#[cfg(test)]` at the bottom of `halfband.rs` since both tests only
  exercise `HalfBand` — **prefer that** (co-locate with the code under
  test) over a separate `tests.rs`, avoiding an extra file for ~30 lines.

## 3. Re-export / public API surface
`convert/mod.rs` is what the rest of `qbz-dsd` (and any downstream crate,
e.g. the playback pipeline that plays DSD files) imports through today via
`qbz_dsd::convert::{DsdPcmConverter, OUTPUT_RATE, DEFAULT_GAIN_DB}` or
`crate::convert::{...}`. Re-export these three names unchanged from
`convert/mod.rs`.

## 4. Tricky coupling to watch
- `DsdPcmConverter::new()` computes `n_stages` (number of half-band ÷2
  stages) from the input DSD rate and allocates
  `stages: Vec<Vec<HalfBand>>` (`stages[stage][channel]`) — this nested
  structure is walked in `next_block()`'s per-channel loop
  (`for stage in self.stages.iter_mut()`); keep `HalfBand` and the
  `stages` field's shape exactly as-is when splitting `converter.rs` from
  `halfband.rs` — only the TYPE definition and its `process()` method move,
  not the nesting logic in `next_block`.
- `next_block()` has a **recursive self-call** (`return self.next_block();`)
  for the "filters still priming, no output yet" case — this recursion is
  bounded by file size but is easy to lose track of if the method is later
  further split into "read" vs. "process" vs. "emit" helper functions; keep
  the recursion in one place (`converter.rs`) rather than spreading it
  across files, so the control flow stays easy to trace.
- The EOF silence-padding branch (`missing = self.total_frames -
  self.frames_emitted`) and the final `frames_emitted >= total_frames =>
  finished = true` bookkeeping are both there to guarantee the WAV
  container's declared frame count exactly matches what's emitted — do not
  extract this bookkeeping into `downmix.rs`; it's converter-lifecycle
  state, not downmix math.
- The multichannel channel-index mapping comment (`3ch = FL FR C · 4ch = FL
  FR BL BR · ...`) documents the DSF/DFF positional channel order — carry
  this comment into `downmix.rs` verbatim since it's the load-bearing
  rationale for the `match self.channels { 3 => ..., 4 => ..., ... }` index
  choices.

## 5. What to verify after the real split
- `cargo test -p qbz-dsd` — both half-band tests (`halfband_dc_gain_is_
  unity`, `halfband_output_count_is_half_input_across_calls`) stay green;
  they're pure numerical tests with no I/O so any refactor-induced numeric
  drift will show immediately.
- `cargo build -p qbz-dsd` and `cargo build --workspace` to confirm whatever
  crate constructs `DsdPcmConverter` (the DSD playback path) still compiles.
- If feasible, a manual smoke test: convert a real DSD64 file end-to-end and
  confirm the output frame count matches `total_frames()` and there's no
  audible glitch at block boundaries (the recursive priming path and the
  EOF padding are the two spots most likely to regress silently).
