use std::sync::Arc;
use std::time::Duration;

use qbz_core::FrontendAdapter;
use qbz_models::Quality;

use super::advance::queue_snapshot;
use super::decision::{advance_state, plan_tick, DriverState};
use super::shell_dispatch::execute_actions;
use super::TICK_MS;
use crate::shell::AppRuntime;

/// Host-supplied side channels the shell drives on each relevant action. Kept
/// as trait objects so qbzd can wire daemon-shared latching / tick timestamping
/// / the QConnect report signal without this module depending on qbzd.
#[derive(Clone)]
pub struct DriverDeps {
    /// Resolve the streaming quality at play time (qbzd passes the daemon prefs).
    pub quality: Arc<dyn Fn() -> Quality + Send + Sync>,
    /// Report-edge signal (T10 wires the QConnect renderer report).
    pub on_edge: Arc<dyn Fn() + Send + Sync>,
    /// Latch a drained error under a category ("stream" | "transport" | "auth").
    pub on_latch: Arc<dyn Fn(&str, String) + Send + Sync>,
    /// Called at the end of every tick (qbzd timestamps `driver_last_tick`).
    pub on_tick: Arc<dyn Fn() + Send + Sync>,
}

/// The 450 ms IO shell. Each tick: read the player event, drain the stream-error
/// latch, project the queue, `plan_tick`, execute the actions, then
/// `advance_state`. Breaks when `shutdown` flips to `true`; the loop is thin by
/// design (01 §3.2 — too thin to hide bugs). Runs safely from boot regardless of
/// auth: with no session the queue is empty and every tick is a near-no-op.
pub async fn run_driver<A: FrontendAdapter + Send + Sync + 'static>(
    runtime: Arc<AppRuntime<A>>,
    deps: DriverDeps,
    mut shutdown: tokio::sync::watch::Receiver<bool>,
) {
    let mut state = DriverState::default();
    let mut ticker = tokio::time::interval(Duration::from_millis(TICK_MS));
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            _ = ticker.tick() => {}
            _ = shutdown.changed() => {
                if *shutdown.borrow() {
                    break;
                }
                continue;
            }
        }

        let core = runtime.core();
        let player = core.player();
        let ev = player.get_playback_event();
        // Drain-once stream-error message (playback.rs:4111).
        let stream_error = player.state.take_stream_error_message();
        let queue = queue_snapshot(core).await;

        let actions = plan_tick(&state, &ev, &queue, stream_error.as_deref());

        execute_actions(&runtime, core, &player, &queue, &deps, &actions).await;

        state = advance_state(&state, &ev, &actions);
        (deps.on_tick)();
    }
    log::info!("[qbzd] driver: shutting down");
}
