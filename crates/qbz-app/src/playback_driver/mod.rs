// crates/qbz-app/src/playback_driver/mod.rs — the headless playback orchestrator.
//
// This re-hosts, as a PURE decision function plus a thin IO shell, the playback
// bookkeeping that today only exists inside the desktop's 450 ms poll loop
// (`crates/qbz/src/playback.rs::start_poll_loop`): end-of-track detection,
// auto-advance, gapless pre-queue, stop-after, seamless-transition cursor sync
// and the periodic session-position save. Every branch below cites the exact
// desktop line it mirrors — the desktop is the reference for tie-breaks, the
// unit tests pin the observable contract (01-architecture.md §3.2).
//
// Split of concerns:
//   * `decision::plan_tick`     — side-effect-free: (state, event, queue, error) → actions
//   * `decision::advance_state` — the pure state-update rule the shell applies each tick
//   * `decision::next_playable` — bounded unstreamable skip-walk (mirrors
//                        `playback.rs::advance_to_playable`, capped at
//                        `MAX_OFFLINE_SKIPS`)
//   * `shell::run_driver`     — the 450 ms IO shell that reads the player, calls
//                        `plan_tick`, and executes the resulting actions
//   * `advance::advance_and_play` — the full advance ritual (skip-walk → play → prefetch
//                        → persist), reused verbatim by the CLI next/prev routes

mod advance;
mod decision;
mod session;
mod session_convert;
mod shell;
mod shell_dispatch;
#[cfg(test)]
mod tests;

/// Poll cadence — the same 450 ms the desktop loop uses
/// (`playback.rs:4088`).
const TICK_MS: u64 = 450;

/// Session-position save cadence: every ~11 ticks ≈ 5 s
/// (`playback.rs:4306`, `save_pos_tick % 11 == 0`).
const SAVE_POSITION_EVERY_N_TICKS: u64 = 11;

/// QConnect report cadence while playing: every ~4 ticks ≈ 2 s
/// (`playback.rs:4069`, `QCONNECT_REPORT_EVERY_N_TICKS`).
const QCONNECT_REPORT_EVERY_N_TICKS: u64 = 4;

/// Bounded skip-walk ceiling for unavailable tracks (Tauri #467 parity;
/// `playback.rs:226`, `MAX_OFFLINE_SKIPS = 5`).
const MAX_OFFLINE_SKIPS: usize = 5;

pub use advance::advance_and_play;
pub use decision::{
    advance_state, next_playable, plan_tick, quality_from_key, DriverAction, DriverState,
    LastTick, QueueSnapshot,
};
pub use session::{restore_session_paused, save_session_now};
pub use shell::{run_driver, DriverDeps};
