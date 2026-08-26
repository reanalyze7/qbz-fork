//! "Stop after this song" marker.

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
}
