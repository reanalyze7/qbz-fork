# crates/qbz/src/visualizer.rs (304 lines)

## Summary
ImmersiveView audio-visualizer glue: a shared latest-wins frame-cell store
(`VizCells`) fed by an FFT producer thread, and a single `install()`
function that wires persistent Slint `VecModel`s, a ~30fps UI-thread drain
timer, and a `set-enabled` handler that starts/stops both the timer and the
producer thread.

## Proposed split
By responsibility — the shared cell/sink types vs the `install()`
orchestration vs the drain-timer closure body (the largest chunk), as a
`visualizer/` directory module:

- `visualizer/mod.rs` (~50 lines) — module doc (lines 1-18), `pub use
  install`, imports.
- `visualizer/cells.rs` (~45 lines) — `VizCells` struct + `SlintVizSink` +
  its `VizSink` impl (lines 34-59). Small, self-contained, no Slint UI
  dependency beyond the model types it eventually feeds.
- `visualizer/install.rs` (~90 lines) — the `install()` function's setup
  half (lines 71-127): tap resolution, persistent `VecModel` creation +
  binding to `VisualizerState`, producer-thread spawn, and the `timer :=
  slint::Timer::default()` + closure-local state variable declarations
  (the `let mut last_*` block) up to `timer.start(...)`.
- `visualizer/drain.rs` (~150 lines) — the drain-timer closure body itself
  (lines 127-271): the paused-gate check, cell-take/`set_row_data` fan-out
  for bars/energy/spectral/waveform/transient, and the WGPU underlay
  shader-frame render block (bands8/level/phase/spectral-ribbon logic).
  Extract as a free function taking every closure-captured variable either
  by `&mut` (for `last_*` state) or by reference (for `weak`, `cells`,
  models, `fft_thread_drain`) — Rust closures can call a named function
  with explicit params instead of inlining the body, keeping `install.rs`'s
  `timer.start(...)` call a one-line dispatch to `drain::tick(&mut state,
  ...)`.
- `visualizer/install.rs` also keeps the tail (lines 272-304): `timer.stop()`
  + `DRAIN_TIMER` stash + the `on_set_enabled` handler registration. Could
  alternatively move `DRAIN_TIMER` thread_local (lines 61-66) into its own
  tiny `visualizer/timer_cell.rs` if `install.rs` still runs long — check
  actual line count after the drain extraction before deciding.

## Re-export surface
`visualizer/mod.rs` stays the public surface: `pub use install::install;`
(or the function is defined directly in `mod.rs` if `install.rs` ends up
unnecessary after moving most of its body into `drain.rs`). The crate's
`main.rs`/`lib.rs` line `mod visualizer;` (or wherever `qbz::visualizer` is
declared) needs no change; call sites use `crate::visualizer::install(...)`
unchanged.

## Coupling / watch out
- The drain closure captures a LOT of mutable local state by move
  (`last_tr`, `last_energy`, `last_bars16`, `last_level_smooth`,
  `last_beat`, `last_phase`, `last_track_id`, `last_progress`, `last_peak`,
  `drain_saw_playing`) plus several `Rc`/`Arc` clones (`weak`, `cells`,
  `bars`/`spectral`/`energy`/`waveform` models, `fft_thread_drain`) — if
  extracted into a free function, bundle the `last_*` variables into one
  `struct DrainState { ... }` that `install.rs` owns as a single `let mut
  state = DrainState::default();` and passes `&mut state` into the drain
  function each tick; this is cleaner than an 9-argument function
  signature and avoids accidentally dropping one variable's persistence
  across ticks.
- `fft_thread` is cloned twice (`fft_thread_drain` for the drain closure's
  unpark-on-resume-play edge, and the original moves into the
  `on_set_enabled` handler) — both clones must keep working after the
  split; don't accidentally consume the only remaining handle in one file.
- `DRAIN_TIMER` thread_local is read/written from BOTH the tail of
  `install()` (stash right after `timer.start()`) and the `on_set_enabled`
  handler (restart/stop) — these must stay in the same file (or the
  thread_local itself in a shared location both can `use`) since it's
  UI-thread-only global state coordinating the timer lifecycle.
- The WGPU underlay block inside the drain tick reads `ImmersiveState`'s
  `app_shader_mode` and calls `crate::shader_underlay::{FrameAudio,
  render_frame}` — a real dependency on a sibling module (`shader_underlay`,
  not in this batch) — keep the exact mode-branching (`m == 4` ribbon vs
  general shader) logic intact; it's tightly tied to the `last_peak`/
  `last_track_id`/`last_progress` reset-detection state.

## Verify after split
- `cargo build -p qbz`.
- `cargo test -p qbz` (no inline tests in this file currently; verify no
  regression in any integration test that touches the visualizer/immersive
  path).
- `cargo clippy -p qbz`.
- Manually smoke-test (this is UI-thread timer/audio-reactive code, hard to
  unit-test): open ImmersiveView, verify the FFT bars/spectral/energy/
  waveform models animate, verify the paused/resume edge (pause playback,
  confirm the visualizer freezes; resume, confirm it un-parks promptly),
  and verify each background-shader mode (especially mode 4, the
  spectral-ribbon with its playback-progress reset-on-seek behavior) still
  renders.
