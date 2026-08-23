//! Network-track gapless pre-queue: fetch bytes (L1/L2 cache -> offline ->
//! network) and hand them to `Player::play_next`.

use super::super::super::quality::local_playback_quality;
use super::super::super::Runtime;
use crate::AppWindow;

/// Spawn the fetch-and-hand-off task for `next_id`, the upcoming NETWORK
/// (non-local) queue track.
pub(super) fn spawn_fetch(runtime: &Runtime, weak: &slint::Weak<AppWindow>, next_id: u64) {
    let runtime = runtime.clone();
    let weak = weak.clone();
    tokio::spawn(async move {
        // Shared tier-walk: L1/L2 (player cache) -> offline
        // -> network, then hand the bytes to play_next.
        let offline = crate::offline::get().await;
        let sink = crate::offline_cache::row_sink(weak.clone());
        if let Some(data) = runtime
            .core()
            .fetch_for_gapless_resolved(next_id, local_playback_quality().0, offline.as_deref(), Some(&sink))
            .await
        {
            let player = runtime.core().player();
            if let Err(e) = player.play_next(data, next_id) {
                log::warn!("[qbz-slint] [GAPLESS] play_next {next_id} failed: {e}");
            } else {
                log::info!("[qbz-slint] [GAPLESS] queued track {next_id} for gapless");
            }
        }
    });
}
