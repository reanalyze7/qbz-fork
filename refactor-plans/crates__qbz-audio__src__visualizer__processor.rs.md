# crates/qbz-audio/src/visualizer/processor.rs (361 lines)

## Summary
Frontend-agnostic FFT/visualizer producer thread: reads from a lockless ring
buffer, computes five typed frame streams (16-bar spectrum, waveform, 512-band
spectral ribbon, 5 energy bands, transient detection) at target FPS, and hands
each to a `VizSink` trait object — the shared DSP core behind both the Tauri
and Slint frontends.

## Proposed split
Clean pure/IO-adjacent boundary: the `VizFrame`/`VizSink` types and thread
spawn are the "API", the main loop is one big stateful function, and
`map_to_log_bars` is a pure helper with its own test. Split by responsibility:

- `visualizer/processor/mod.rs` (~90 lines) — module doc, constants
  (`NUM_ENERGY_BANDS`, `NUM_SPECTRAL_BANDS`, `SPECTRAL_UPDATE_RATE_HZ`,
  `SPECTRAL_SMOOTHING`, `ENERGY_BAND_RANGES`), `VizFrame` enum, `VizSink`
  trait, `IDLE_POLL` const, `spawn_visualizer_thread` — this is the complete
  public API surface of the module.
- `visualizer/processor/loop.rs` (~185 lines) — `run_fft_loop` (the entire
  main loop body, lines 95-275) unchanged as one function — it's a single
  cohesive state machine (samples/windowed/output/smoothed buffers, energy
  band state, transient state) that resists further decomposition without
  restructuring the algorithm; keep it as ONE function in its own file rather
  than trying to split the loop body itself (splitting the loop's internals
  into helper functions would multiply the (buffer, state) parameter list
  across call boundaries for no real cohesion gain — the "module" cut is the
  right grain here, not deeper).
- `visualizer/processor/log_bars.rs` (~55 lines) — `map_to_log_bars` (the pure
  log-spaced frequency bar mapping, lines 277-331), which is standalone
  logic already, no shared mutable state.
- `visualizer/processor/tests.rs` (~30 lines) — the `#[cfg(test)] mod tests`
  block (`test_log_frequency_distribution`), declared via `#[cfg(test)] mod
  tests;` in `mod.rs`.

## Re-export surface
`visualizer/processor/mod.rs` re-exports `VizFrame`, `VizSink`,
`spawn_visualizer_thread` at `crate::visualizer::processor::*` (or wherever
`super::VisualizerTap, FFT_SIZE, NUM_BARS, TARGET_FPS` are currently imported
from — check `visualizer/mod.rs`'s existing `pub use processor::...` and keep
it identical) so the Tauri/Slint sink adapters that call
`spawn_visualizer_thread(tap, sink)` are unaffected.

## Coupling / watch out
- `run_fft_loop` (loop.rs) calls `map_to_log_bars` (log_bars.rs) — needs
  `use super::log_bars::map_to_log_bars;` (or re-export it via `mod.rs` and
  `use super::map_to_log_bars;`); keep the function `pub(crate)` or
  `pub(super)` since it's an internal helper, not part of the public API.
- `run_fft_loop` uses `SpectralAnalyzer` (imported from `crate::SpectralAnalyzer`
  at the crate root) and `super::{VisualizerTap, FFT_SIZE, NUM_BARS,
  TARGET_FPS}` from the parent `visualizer` module — both imports need to
  change from `super::` to `super::super::` (or `crate::visualizer::...`)
  once `loop.rs` is one directory deeper than the original `processor.rs`.
- The five `NUM_ENERGY_BANDS`/`NUM_SPECTRAL_BANDS`/etc. consts and
  `ENERGY_BAND_RANGES` array are used ONLY inside `run_fft_loop` — could move
  them into `loop.rs` instead of `mod.rs` since nothing else references them;
  recommend keeping them in `mod.rs` anyway since they read as "module-level
  tuning constants" a maintainer would look for near the top, but either
  placement compiles fine.
- The `IDLE_POLL` doc comment is fairly long and references the `unpark()`
  wake path used by the Slint frontend — keep that comment attached to the
  const wherever it lands (mod.rs), since it's the kind of behavioral
  contract note easy to lose in a mechanical split.

## Verify after split
- `cargo test -p qbz-audio visualizer::processor::` —
  `test_log_frequency_distribution` stays green.
- `cargo check -p qbz-audio` and grep for
  `visualizer::processor::spawn_visualizer_thread` / `VizSink` / `VizFrame`
  importers (the Tauri viz-event adapter and the Slint frame-latch adapter)
  to confirm the public path and trait/enum shapes are unchanged.
- Manually confirm (or via existing integration test if any) that the FFT
  loop still paces at `TARGET_FPS` and idles correctly on
  disabled/paused — this file has real timing behavior that a pure
  compile-check won't catch.
