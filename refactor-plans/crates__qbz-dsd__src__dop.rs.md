# crates/qbz-dsd/src/dop.rs (266 lines)

## Summary
DoP (DSD-over-PCM) framing per the DoP Open Standard v1.1: a stateful frame packer
(`DopPacker`) that interleaves DSD bits into S32 PCM words with an alternating
marker byte, and a whole-file streaming iterator (`DopStream`) that pulls DSD bytes
from a demuxer, bit-reverses when needed, and yields packed S32 samples — plus unit
tests (including a simulated mid-stream I/O error test).

## Proposed split
By concern (packer vs stream iterator vs tests) — this is a small, cohesive audio
file; the split is mostly about isolating the two independently-testable pieces and
carving out the sizeable test modules.

- `dop/mod.rs` (~20 lines) — module doc, `dop_carrier_rate` const fn,
  `pub use` re-exports of `DopPacker` and `DopStream` so `crate::dop::X` paths (used
  by the ALSA/output backend that consumes `DopStream`) are unchanged.
- `dop/packer.rs` (~50 lines) — `DopPacker` struct + `new`/`pack`/`silence`/`Default`.
- `dop/stream.rs` (~90 lines) — `DopStream` struct, `REFILL_BYTES_PER_CH` const,
  `new`/`io_error`/`carrier_rate`/`dsd_rate`/`total_frames`/`refill`, and the
  `Iterator` impl.
- `dop/tests.rs` (~35 lines) — the first `#[cfg(test)] mod tests` block
  (`packer_layout_and_marker_alternation`, `silence_is_0x69_payload_with_markers`),
  referencing `DopPacker` via `use super::packer::*;`.
- `dop/io_error_tests.rs` (~60 lines) — the second `#[cfg(test)] mod io_error_tests`
  block (the `FailAfter` mock `DsdDemuxer` + `demux_io_error_sets_sticky_flag_not_clean_eof`),
  referencing `DopStream` via `use super::stream::*;` and `crate::demux::{DsdDemuxer,
  DsdError, DsdStreamInfo}`.

## Re-export surface
`dop/mod.rs` re-exports `dop_carrier_rate`, `DopPacker`, `DopStream` so every existing
`use crate::dop::{DopStream, DopPacker, dop_carrier_rate};` (or `qbz_dsd::dop::X` from
another crate) call site is unaffected.

## Coupling / watch out
- `DopStream::new` and `refill` depend on `crate::demux::{DsdDemuxer, DsdError}` and
  `crate::dsd2pcm::bit_reverse` — both need re-importing in `stream.rs`.
- `dop_carrier_rate` (a `const fn` in `mod.rs`) is called both by `DopStream::carrier_rate`
  (stream.rs) — cross-module call, keep it `pub(crate)` or `pub` at the `dop` module
  root so `super::dop_carrier_rate` resolves from `stream.rs`.
- `DopStream` owns a `Box<dyn DsdDemuxer>` plus internal buffer/index/done/io_error
  state — this is entirely self-contained within the struct, no shared globals, so
  the split carries no hidden state risk.
- The two test modules are independent of each other (one tests the pure packer, the
  other tests the stream's I/O-error path via a mock demuxer) — safe to split into
  separate files.

## Verify after split
- `cargo test -p qbz-dsd dop` — all 3 existing tests must stay green (packer layout,
  silence payload, sticky I/O error flag).
- `cargo check -p qbz-dsd` for the ALSA/output backend that constructs `DopStream`
  and iterates it for DoP playback.
- Manual/smoke test: play a DSD file over a DoP-capable output path and confirm no
  audible marker-sequence corruption (loud noise would indicate a broken marker
  alternation — a regression this split must not introduce).
