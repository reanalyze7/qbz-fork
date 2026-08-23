//! `refresh` / `refresh_with_favorites` entry points, and the
//! `refresh_async` orchestration that ties the snapshot pull, coverflow
//! build, and event-loop push together.

mod apply;
mod coverflow;
mod snapshot;

use qbz_app::settings::playback::AutoplayMode;

use super::QueueController;

impl QueueController {
    /// Pull a fresh full queue snapshot, re-apply the search filter and
    /// current page, and push the result onto `QueueState`. Spawns on the
    /// tokio runtime; safe to call from any thread.
    pub fn refresh(&self) {
        let this = self.clone();
        self.handle.spawn(async move {
            this.refresh_async().await;
        });
    }

    /// Refresh the queue view; when online, first re-pull the SHARED
    /// favorite cache from the network (used after a fresh play starts, so
    /// hearts reflect cross-device changes — same cadence as before, but
    /// now feeding `fav_cache` + its disk mirror instead of a queue-local
    /// set). Offline, the disk-seeded cache is used as-is.
    pub fn refresh_with_favorites(&self) {
        let this = self.clone();
        self.handle.spawn(async move {
            if !crate::offline_mode::engine().is_offline() {
                match this.runtime.core().favorite_track_ids().await {
                    Ok(ids) => {
                        // set_all mirrors to disk (blocking rusqlite).
                        let _ =
                            tokio::task::spawn_blocking(move || crate::fav_cache::set_all(ids))
                                .await;
                    }
                    Err(e) => {
                        log::warn!("[qbz-slint] queue: favorite_track_ids failed: {e}");
                    }
                }
            }
            this.refresh_async().await;
        });
    }

    pub(super) async fn refresh_async(&self) {
        let perf_start = std::time::Instant::now();

        let rows = self.pull_snapshot_rows().await;
        let cf = self.build_coverflow(&rows.history, rows.current_track.as_ref(), &rows.upcoming);

        // --- Infinite-play flag ------------------------------------------
        let infinite = self
            .playback
            .get_preferences()
            .map(|p| p.autoplay_mode == AutoplayMode::InfiniteRadio)
            .unwrap_or(false);

        self.push_to_ui(rows, cf, infinite);

        log::debug!(
            "[coverflow-perf] refresh_async total={}ms",
            perf_start.elapsed().as_millis()
        );
    }
}
