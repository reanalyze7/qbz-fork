//! "Stop after this song" marker + the infinite-play status read.

use qbz_app::settings::playback::AutoplayMode;

use super::QueueController;

impl QueueController {
    /// Toggle the "stop after this song" marker on the queue track with `id`
    /// (a decimal string, matching `QueueItem.id`). Idempotent — tapping the same
    /// track clears it. The marker auto-clears on queue mutation inside the core,
    /// and `refresh_async` reflects the current marker into `QueueState.stop-after-id`.
    pub fn toggle_stop_after(&self, id: String) {
        let Ok(track_id) = id.parse::<u64>() else {
            return;
        };
        let this = self.clone();
        self.handle.spawn(async move {
            let already = this.runtime.core().get_stop_after().await == Some(track_id);
            if already {
                this.runtime.core().clear_stop_after().await;
            } else {
                this.runtime.core().set_stop_after(track_id).await;
            }
            this.refresh_async().await;
        });
    }

    /// Whether `InfiniteRadio` autoplay is currently on. Reads the same
    /// playback preference the toggle and the sidebar flag use. NOTE: the
    /// queue no longer actually refills on this setting — `try_infinite_refill`
    /// in `playback.rs` always returns `false` since the `qbz-radio` crate it
    /// depended on was removed (REMOVAL-SPEC.md §6 "Radio"); the setting/UI
    /// stayed in place as a separate, unresolved follow-up.
    pub fn is_infinite_play(&self) -> bool {
        self.playback
            .get_preferences()
            .map(|p| p.autoplay_mode == AutoplayMode::InfiniteRadio)
            .unwrap_or(false)
    }
}
