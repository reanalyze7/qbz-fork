# crates/qbz-app/src/playback_driver.rs (893 lines)

## Summary
The headless (qbzd daemon) playback orchestrator: a pure per-tick decision
function (`plan_tick`/`advance_state`) that mirrors the desktop's 450 ms
poll loop, plus the IO shell (`run_driver`) that drives it and the shared
"advance ritual" (`advance_and_play`) and session persistence helpers reused
by the CLI next/prev routes.

## Proposed split
The file already documents its own split of concerns in the header comment
— follow that boundary plus pull out session persistence and tests:

- `playback_driver/mod.rs` (~20 lines) — module doc (verbatim from the
  current header), `pub use` re-exports of everything currently public
  (`DriverAction`, `LastTick`, `DriverState`, `QueueSnapshot`, `DriverDeps`,
  `next_playable`, `plan_tick`, `advance_state`, `quality_from_key`,
  `run_driver`, `advance_and_play`, `save_session_now`,
  `restore_session_paused`), and the `TICK_MS`/`SAVE_POSITION_EVERY_N_TICKS`/
  `QCONNECT_REPORT_EVERY_N_TICKS`/`MAX_OFFLINE_SKIPS` constants (small
  enough to keep at the top level, or move into `decision.rs`).
- `playback_driver/decision.rs` (~300 lines) — the PURE core: `DriverAction`,
  `LastTick`, `DriverState`, `QueueSnapshot`, `next_playable`, `plan_tick`,
  `advance_state`, `quality_from_key`. Zero I/O, zero `async` — the ideal
  "pure" module and the one most worth keeping intact as a unit since
  `plan_tick`'s branch-by-branch comments cite exact desktop line numbers.
- `playback_driver/shell.rs` (~200 lines) — the IO shell: `DriverDeps`,
  `run_driver`, `advance_and_play_logged`. Depends on `decision.rs` +
  `advance.rs`.
- `playback_driver/advance.rs` (~110 lines) — `advance_and_play`,
  `prefetch_successors`, `queue_snapshot`. The reusable ritual the CLI
  next/prev routes call directly (per the header comment, T7) — keep this
  as its own file since it has external callers beyond the driver loop.
- `playback_driver/session.rs` (~140 lines) — `save_session_now`,
  `restore_session_paused`, `repeat_to_str`, `repeat_from_str`,
  `to_persisted`, `from_persisted`. The daemon-side session
  persistence/restore + the `QueueTrack`↔`PersistedQueueTrack` conversions.
- `playback_driver/tests.rs` (~150 lines) — the whole `#[cfg(test)] mod
  tests` block (the `ev`/`q` builders + all 11 tests), `use
  super::decision::*;` (or `use super::*;` via the mod.rs re-exports).

## Re-export surface
`playback_driver/mod.rs` is the `mod playback_driver;` target already used
as `qbz_app::playback_driver::X` by qbzd and the CLI next/prev routes. All
current public items must stay reachable at that path via `pub use
decision::*; pub use shell::*; pub use advance::*; pub use session::*;`.

## Coupling / watch out
- `plan_tick`'s in-line comments cite exact `crates/qbz/src/playback.rs`
  line numbers as the ported-from reference (e.g. `playback.rs:4111`,
  `:4305-4308`, `:4324-4371` etc.) — preserve every one of these comments
  verbatim when moving code; they're the audit trail against the desktop
  original, not incidental.
- `advance_state` must stay bit-for-bit paired with `plan_tick` (same file
  or same module) — they're two halves of one state machine and the tests
  exercise them together every time.
- `DriverDeps` closures (`quality`, `on_edge`, `on_latch`, `on_tick`) are
  the seam qbzd wires without depending on this crate — don't collapse them
  into concrete daemon types during the split.
- `advance_and_play` in `advance.rs` is called both by `shell.rs`'s
  `advance_and_play_logged` (auto-advance) and directly by the CLI's
  next/prev route (outside this file entirely) — it must stay `pub`.
- `MAX_OFFLINE_SKIPS` is used both in `decision.rs` (`next_playable` bound)
  and `advance.rs` (`advance_and_play`'s skip-walk loop) — keep it visible
  to both, e.g. defined once in `mod.rs` or `decision.rs` and imported.

## What to verify after the real split
- `cargo build -p qbz-app` and `cargo build -p qbzd` (the daemon binary
  that actually drives this loop).
- `cargo test -p qbz-app playback_driver::` — all 11 existing unit tests
  green (end_edge_advances, mid_track_pause_does_not_advance,
  stop_after_one_shot, gapless_arms_exactly_once,
  repeat_one_advances_instead_of_finishing, queue_finished_when_nothing_
  playable, skip_walk_bounds, position_save_cadence_11_ticks,
  seamless_gapless_transition_syncs_cursor, stream_error_latches,
  duration_zero_never_advances).
- Smoke-test: run qbzd, play a short local track to completion, confirm
  auto-advance + gapless pre-queue + a `qbzd next`/`qbzd prev` CLI call
  still work (exercises `advance.rs` from two call sites).
