# crates/qbz-player/src/player/playback_engine.rs (1039 lines)

## Summary
Unified `PlaybackEngine` enum abstracting 4 audio backends (Rodio, ALSA
Direct, JACK, DoP/DSD-over-PCM) behind one API (`append`, `play`, `pause`,
`stop`, `set_volume`, position/duration, crossfade), plus the 3 long-lived
writer/feeder thread functions that drive ALSA Direct / JACK / DoP gapless
playback.

## Proposed split
By backend (pure/IO split doesn't apply well — every variant IS an IO driver
thread — so split by "which output backend"), keeping the enum + dispatch
`impl` together since match arms need all variants visible:

- `playback_engine/mod.rs` (~120 lines) — module doc, `SourceQueue<S>` (generic
  queue used by all 3 non-Rodio backends, lines 21–71), the `PlaybackEngine`
  enum definition (variant list only, lines 73–129), and re-exports.
- `playback_engine/dispatch.rs` (~330 lines) — the big `impl PlaybackEngine`
  block's cross-backend methods that `match self` over all variants: `append`,
  `play`, `pause`, `stop`/`stop_inner`, `set_volume`, `empty`,
  `take_source_transition`, `position_secs`, `duration_secs`, `is_alsa_direct`,
  `is_dop`, `supports_crossfade`, `Drop`. Still likely over 130 — split further
  by concern: `dispatch/transport.rs` (play/pause/stop/Drop, ~100 lines),
  `dispatch/append.rs` (append/append_dop/crossfade_to, ~130 lines),
  `dispatch/query.rs` (empty/take_source_transition/position/duration/
  is_alsa_direct/is_dop/supports_crossfade, ~100 lines).
- `playback_engine/constructors.rs` (~130 lines) — `new_rodio`, `new_alsa_direct`,
  `new_jack`, `new_alsa_dop` (lines 131–261), each spawning its backend's
  writer/feeder thread.
- `playback_engine/alsa_writer.rs` (~115 lines) — `alsa_writer_thread` (lines
  711–822).
- `playback_engine/jack_feeder.rs` (~95 lines) — `jack_feeder_thread` (lines
  830–919), `#[cfg(target_os = "linux")]`.
- `playback_engine/dop_writer.rs` (~110 lines) — `dop_writer_thread` (lines
  936–1039), `#[cfg(target_os = "linux")]`.

## Re-export surface
`playback_engine/mod.rs` re-exports `pub enum PlaybackEngine` (with all its
methods via the split `impl` blocks — Rust allows multiple `impl Foo` blocks
across files as long as they're all `mod`-included, so `dispatch.rs` etc. just
need `use super::PlaybackEngine;` and `impl PlaybackEngine { ... }` again).
Existing callers do `use crate::player::playback_engine::PlaybackEngine;` (or
however it's currently imported) — path unchanged since `mod.rs` keeps the
same public name.

## Coupling / watch out
- `SourceQueue<S>` is used by ALL THREE thread-based backends with different
  `S` (`BoxedSampleIter` for ALSA/JACK, `BoxedDopIter` for DoP) — keep it in
  `mod.rs` so all thread files can `use super::SourceQueue`.
- The three writer/feeder threads are near-identical in control flow (poll
  queue → play/pause loop → fill buffer → write → handle source-ended →
  gapless transition) but have real behavioral differences (DoP's silence
  padding/native-DSD, JACK's ring-buffer pacing) — resist the urge to
  over-abstract them into one generic function during this split; keep them
  separate files as literal ports, refactor-to-shared-helper is a separate
  future task.
- `stop_inner` is called both by `pub fn stop(self)` and by `impl Drop` — both
  must stay reachable from `dispatch/transport.rs` (or wherever `stop_inner`
  ends up); don't accidentally duplicate the logic.
- `#[cfg(target_os = "linux")]` gates JACK and DoP variants/methods/threads
  throughout — every new file touching those must repeat the same cfg gate at
  the item level (not just module level) to keep non-Linux builds compiling.
- `crossfade_to` only handles `Self::Rodio` and use a `let ... else` early
  return for other variants — keep this method colocated with `append` (both
  are "put a new source into the engine") rather than with `play`/`pause`.

## Verify after split
- `cargo build -p qbz-player` on Linux (all 4 backends) and ideally on a
  non-Linux target/feature check (`cargo check --target x86_64-apple-darwin`
  if cross tooling available, else at minimum grep for cfg consistency).
- `cargo test -p qbz-player` — check for existing tests exercising this module.
- Manual smoke test: play/pause/stop/next-track (gapless transition) on
  whichever backend is configured in dev (likely Rodio or ALSA Direct) via the
  `run` skill / actual app.
