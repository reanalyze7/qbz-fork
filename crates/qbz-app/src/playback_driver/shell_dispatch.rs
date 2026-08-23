use std::sync::Arc;

use qbz_core::{FrontendAdapter, QbzCore};
use qbz_models::Quality;
use qbz_player::Player;

use super::advance::advance_and_play;
use super::decision::{DriverAction, QueueSnapshot};
use super::shell::DriverDeps;
use crate::shell::AppRuntime;

/// Execute one tick's worth of [`DriverAction`]s in order. Split out of
/// `shell::run_driver`'s loop body purely for file-size hygiene — this is
/// still the same per-action dispatch the desktop loop performs inline.
pub(super) async fn execute_actions<A: FrontendAdapter + Send + Sync + 'static>(
    runtime: &Arc<AppRuntime<A>>,
    core: &Arc<QbzCore<A>>,
    player: &Arc<Player>,
    queue: &QueueSnapshot,
    deps: &DriverDeps,
    actions: &[DriverAction],
) {
    for action in actions {
        match action {
            DriverAction::SyncCursorTo(id) => {
                core.sync_current_to_id(*id).await;
            }
            DriverAction::ArmGapless(id) => {
                let quality = (deps.quality)();
                if let Some(bytes) = core.fetch_for_gapless_resolved(*id, quality, None, None).await
                {
                    if let Err(e) = player.play_next(bytes, *id) {
                        log::warn!("[qbzd] driver: gapless play_next failed: {e}");
                    }
                }
            }
            DriverAction::PauseStopAfter => {
                // The ended track is the queue's current track (playback.rs:4708).
                let finished = queue.current;
                if finished != 0 && core.consume_stop_after_if(finished).await {
                    if let Err(e) = core.pause() {
                        log::warn!("[qbzd] driver: stop-after pause failed: {e}");
                    }
                } else {
                    // Marker cleared between the snapshot and the consume — fall
                    // through to the normal advance (desktop parity: stop-after
                    // and advance share one track-end block).
                    advance_and_play_logged(runtime, (deps.quality)()).await;
                }
            }
            DriverAction::AdvanceAndPlay => {
                advance_and_play_logged(runtime, (deps.quality)()).await;
            }
            DriverAction::SavePosition(p) => {
                runtime.with_session_store(|s| {
                    if let Err(e) = s.save_position(*p) {
                        log::debug!("[qbzd] driver: save_position failed: {e}");
                    }
                });
            }
            DriverAction::LatchError(m) => {
                (deps.on_latch)("stream", m.clone());
            }
            DriverAction::ReportEdge => {
                (deps.on_edge)();
            }
            DriverAction::QueueFinished => {
                if queue.autoplay_infinite {
                    log::info!(
                        "[qbzd] driver: autoplay 'infinite' unsupported on qbzd v1, \
                         treated as queue-finished"
                    );
                }
                log::info!("[qbzd] driver: queue finished");
                if let Err(e) = core.stop() {
                    log::warn!("[qbzd] driver: stop on queue-finished failed: {e}");
                }
            }
        }
    }
}

/// Run the advance ritual and log its outcome (queue-finished on `Ok(None)`, the
/// error on `Err`). The daemon stops on a genuine queue edge just like the
/// pure-decision `QueueFinished` branch does. Always forward — the driver's
/// auto-advance on track-end never walks backward (reverse is the `qbzd prev`
/// route's concern, wired directly through `advance_and_play(..., false)`).
async fn advance_and_play_logged<A: FrontendAdapter + Send + Sync + 'static>(
    runtime: &Arc<AppRuntime<A>>,
    quality: Quality,
) {
    match advance_and_play(runtime, quality, true).await {
        Ok(Some(_)) => {}
        Ok(None) => {
            log::info!("[qbzd] driver: advance found nothing playable — queue finished");
            if let Err(e) = runtime.core().stop() {
                log::warn!("[qbzd] driver: stop after empty advance failed: {e}");
            }
        }
        Err(e) => log::warn!("[qbzd] driver: advance failed: {e}"),
    }
}
