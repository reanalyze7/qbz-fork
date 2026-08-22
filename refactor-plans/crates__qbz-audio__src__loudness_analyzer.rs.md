# crates/qbz-audio/src/loudness_analyzer.rs (252 lines)

## Summary
Background thread that receives decoded audio samples via a channel,
computes EBU R128 integrated LUFS with the `ebur128` crate, and writes a
shared `Arc<AtomicU32>` gain value for `DynamicAmplify` to read — with an
initial measurement after ~10s and periodic refinement every ~5s thereafter,
cache-aware.

## Proposed split
By responsibility (public thread-spawning API vs internal per-track analyzer
state vs the pure gain-math helper):

- `loudness_analyzer/mod.rs` (~45 lines) — lines 1-43: module doc,
  `MAX_GAIN_DB` const, `pub struct LoudnessAnalyzer` + its `spawn()` fn (the
  public API — thread spawn wrapper only).
- `loudness_analyzer/run_loop.rs` (~75 lines) — lines 45-116: the `run()`
  associated fn (the message-receive loop: `NewTrack`/`Samples`/`Reset`/
  `Shutdown` handling) — this is the thread body, kept separate from the
  `spawn()` wrapper since it's the actual control-flow logic.
- `loudness_analyzer/state.rs` (~90 lines) — lines 118-169: `AnalyzerState`
  struct + `new()` + `reset_analyzer()` (construction/reset only — the pure
  per-track state and its lifecycle).
- `loudness_analyzer/measure.rs` (~80 lines) — lines 171-246: `feed_samples`
  and `measure_and_update` (the actual EBU R128 feeding + measurement +
  cache-write logic) as further `impl AnalyzerState` methods in a separate
  file from the constructor.
- `loudness_analyzer/gain_math.rs` (~10 lines) — lines 248-252:
  `compute_gain_capped` — the one pure, easily-unit-testable function in the
  whole file (dB-to-linear-gain with the 6dB cap).

## Re-export surface
`loudness_analyzer/mod.rs` stays the `mod loudness_analyzer;` target already
used as `qbz_audio::loudness_analyzer::LoudnessAnalyzer` (its only public
item — `spawn()` is the sole entry point `DynamicAmplify`/the playback
pipeline calls). `AnalyzerState` and `compute_gain_capped` are crate-internal
today (no `pub`) and can stay `pub(crate)` or private across the split via
`mod state; mod measure; mod gain_math;` without `pub use`, as long as
`run_loop.rs` can still reach them via `super::state::AnalyzerState` etc.

## Coupling / watch out
- `AnalyzerState` is constructed in `run_loop.rs`'s `run()` (on `NewTrack`)
  and mutated via `feed_samples`/`reset_analyzer` from the same loop — if
  `state.rs` and `measure.rs` are split, `impl AnalyzerState` blocks in both
  files reference the same struct fine (multiple impl blocks across files in
  one module are fine in Rust), but double-check field visibility: all
  fields are currently private to the file; making them `pub(super)` or
  keeping `state.rs`+`measure.rs`+`run_loop.rs` all under
  `loudness_analyzer/` (same parent module) means default private-to-module
  visibility already covers cross-file access within `loudness_analyzer/`.
- `gain_atomic: Option<Arc<AtomicU32>>` is written from TWO call sites
  (`NewTrack` cache-hit branch in `run_loop.rs`, and `measure_and_update`'s
  "only update on first measurement" branch in `measure.rs`) — the comment
  "Only update the live gain on the FIRST measurement... applying gain
  changes mid-song causes audible volume fluctuations" is a load-bearing
  invariant; keep it attached to `measure_and_update` wherever it lands.
  Refinements MUST continue to update the cache but not the atomic.
- `LoudnessCache` (from `super::loudness_cache`) is read in `run_loop.rs`
  (cache-hit path) and written in `measure.rs` (`cache.set(...)`) — both
  files need the same `use super::loudness_cache::LoudnessCache;` import.
- `db_to_linear` (from `super::loudness`) is used only by
  `compute_gain_capped` — keep that single `use` in `gain_math.rs`.

## Verify after split
- `cargo check -p qbz-audio` and `cargo build -p qbz-audio`.
- `cargo test -p qbz-audio` (check whether any test in the crate references
  `loudness_analyzer::compute_gain_capped` or `AnalyzerState` directly —
  if so, update the path to the new submodule).
- Smoke-test: play a track with normalization enabled, confirm gain still
  converges (initial ~10s then refinement) and a seek doesn't reset gain to
  full/incorrect (the `Reset` message path).
