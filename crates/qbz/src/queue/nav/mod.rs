//! Play/remove callbacks for the upcoming list, plus (in `reorder.rs`) the
//! index-resolution + drag-reorder machinery they share.

mod reorder;

use super::QueueController;

impl QueueController {
    /// Play the upcoming entry at `page_index` within the current page.
    /// Resolves the page-local index to a queue-wide upcoming index,
    /// honoring the active search filter, then jumps the core there.
    pub fn play_upcoming(&self, page_index: usize) {
        let this = self.clone();
        self.handle.spawn(async move {
            let Some(upcoming_index) = this.resolve_upcoming_index(page_index).await else {
                log::warn!("[qbz-slint] queue: play_upcoming {page_index} out of range");
                return;
            };
            let Some(track) = this.runtime.core().play_upcoming_at(upcoming_index).await else {
                log::warn!("[qbz-slint] queue: play_upcoming_at {upcoming_index} miss");
                return;
            };
            crate::playback::after_track_change(&this.runtime, &this.weak, track.id).await;
            this.refresh_async().await;
        });
    }

    /// Play an upcoming track by its QUEUE-WIDE (unfiltered) index. The immersive
    /// coverflow lists `state.upcoming.take(3)` regardless of the sidebar's page
    /// or search, so its cards must NOT go through `play_upcoming`'s page-local
    /// `resolve_upcoming_index` (that would play the wrong track when the sidebar
    /// is paged/filtered). History is already queue-wide via `play_history`.
    pub fn play_coverflow_upcoming(&self, upcoming_index: usize) {
        let this = self.clone();
        self.handle.spawn(async move {
            let Some(track) = this.runtime.core().play_upcoming_at(upcoming_index).await else {
                log::warn!("[qbz-slint] queue: play_coverflow_upcoming {upcoming_index} miss");
                return;
            };
            crate::playback::after_track_change(&this.runtime, &this.weak, track.id).await;
            this.refresh_async().await;
        });
    }

    /// Remove the upcoming entry at `page_index` within the current page.
    pub fn remove_upcoming(&self, page_index: usize) {
        let this = self.clone();
        self.handle.spawn(async move {
            let Some(upcoming_index) = this.resolve_upcoming_index(page_index).await else {
                log::warn!("[qbz-slint] queue: remove_upcoming {page_index} out of range");
                return;
            };
            this.runtime.core().remove_upcoming_track(upcoming_index).await;
            this.refresh_async().await;
        });
    }

    /// Remove every upcoming track after the row at page-local `page_index`
    /// (that row is kept). Resolves the page-local index to a queue-wide
    /// upcoming index first (honoring the search filter), then truncates the
    /// upcoming list in play order. Mirrors `remove_upcoming`'s handling.
    pub fn remove_all_after(&self, page_index: usize) {
        let this = self.clone();
        self.handle.spawn(async move {
            let Some(upcoming_index) = this.resolve_upcoming_index(page_index).await else {
                log::warn!("[qbz-slint] queue: remove_all_after {page_index} out of range");
                return;
            };
            this.runtime.core().remove_upcoming_after(upcoming_index).await;
            this.refresh_async().await;
        });
    }
}
