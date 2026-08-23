//! Save-as-playlist / add-to-playlist handoffs to the playlist picker.

use super::QueueController;

impl QueueController {
    /// Open the Add-to-Playlist picker seeded with the queue's tracks
    /// (current + upcoming, de-duplicated, in play order). The picker's inline
    /// "Create new playlist" row turns the queue into a named playlist; picking
    /// an existing one appends the queue to it. Mirrors Tauri's
    /// handleSaveQueueAsPlaylist (which reuses the add-to-playlist modal).
    pub fn save_as_playlist(&self) {
        let this = self.clone();
        self.handle.spawn(async move {
            let state = this.runtime.core().get_queue_state_full().await;
            let mut ids: Vec<u64> = Vec::new();
            if let Some(curr) = state.current_track.as_ref() {
                ids.push(curr.id);
            }
            for t in state.upcoming.iter() {
                if !ids.contains(&t.id) {
                    ids.push(t.id);
                }
            }
            if ids.is_empty() {
                return;
            }
            let ids_str: Vec<String> = ids.iter().map(|id| id.to_string()).collect();
            let runtime = this.runtime.clone();
            let handle = this.handle.clone();
            let _ = this.weak.upgrade_in_event_loop(move |w| {
                crate::playlist_picker::open_for_ids(&w, runtime, &handle, ids_str, false);
            });
        });
    }

    /// Open the Add-to-Playlist picker seeded with a single upcoming row (the
    /// track at page-local `page_index`). Reuses `save_as_playlist`'s picker
    /// handoff with just that one track — matching the per-track "Add to
    /// playlist" action in the album/track menus.
    pub fn add_to_playlist(&self, page_index: usize) {
        let this = self.clone();
        self.handle.spawn(async move {
            let Some(upcoming_index) = this.resolve_upcoming_index(page_index).await else {
                log::warn!("[qbz-slint] queue: add_to_playlist {page_index} out of range");
                return;
            };
            let state = this.runtime.core().get_queue_state_full().await;
            let Some(track) = state.upcoming.get(upcoming_index) else {
                log::warn!("[qbz-slint] queue: add_to_playlist {page_index} -> no upcoming track");
                return;
            };
            let ids_str = vec![track.id.to_string()];
            let runtime = this.runtime.clone();
            let handle = this.handle.clone();
            let _ = this.weak.upgrade_in_event_loop(move |w| {
                crate::playlist_picker::open_for_ids(&w, runtime, &handle, ids_str, false);
            });
        });
    }
}
