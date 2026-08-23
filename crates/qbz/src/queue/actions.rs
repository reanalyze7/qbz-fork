//! Misc per-queue actions: play-history, clear, favorite/infinite-play/
//! stop-after toggles.

use qbz_app::settings::playback::AutoplayMode;

use super::QueueController;

impl QueueController {
    /// Play a History entry by its `index` in the history list.
    pub fn play_history(&self, index: usize) {
        let this = self.clone();
        self.handle.spawn(async move {
            let state = this.runtime.core().get_queue_state_full().await;
            let Some(track) = state.history.get(index).cloned() else {
                log::warn!("[qbz-slint] queue: play_history {index} out of range");
                return;
            };
            // History plays start a fresh single-track queue, matching the
            // Tauri handlePlayHistoryTrack path (the history list is not a
            // re-entry point into the existing queue order).
            this.runtime.core().set_queue(vec![track.clone()], Some(0)).await;
            crate::playback::after_track_change(&this.runtime, &this.weak, track.id).await;
            this.refresh_async().await;
        });
    }

    /// Empty the queue. When nothing is playing the now-playing slot is
    /// wiped too, mirroring the Tauri `handleClearQueue` behaviour.
    pub fn clear(&self) {
        let this = self.clone();
        self.handle.spawn(async move {
            let playing = this.runtime.core().get_playback_state().is_playing;
            // keep_current = playing: keep the slot only while audible.
            this.runtime.core().clear_queue(playing).await;
            if let Ok(mut view) = this.view.lock() {
                view.page = 0;
                view.search.clear();
            }
            this.refresh_async().await;
        });
    }

    /// Toggle the favorite state of the now-playing track.
    pub fn toggle_favorite(&self) {
        let this = self.clone();
        self.handle.spawn(async move {
            // Offline = read-only hearts (spec 4.3).
            if crate::offline_mode::engine().is_offline() {
                crate::toast::info_weak(&this.weak, qbz_i18n::t("Not available offline"));
                return;
            }
            let state = this.runtime.core().get_queue_state_full().await;
            let Some(track) = state.current_track else {
                return;
            };
            let make_favorite = !crate::fav_cache::contains(track.id);
            match this.runtime.core().set_track_favorite(track.id, make_favorite).await {
                Ok(()) => {
                    // Keep the shared cache (memory + disk) in sync so every
                    // other heart surface reflects the change immediately.
                    crate::fav_cache::set(track.id, make_favorite);
                    // reco: log a favorite ADD (skip un-favorite) for scoring.
                    if make_favorite {
                        let tid = track.id;
                        tokio::task::spawn_blocking(move || {
                            crate::reco::log_favorite_track(tid, None, None)
                        });
                    }
                }
                Err(e) => {
                    log::error!("[qbz-slint] queue: toggle favorite failed: {e}");
                }
            }
            this.refresh_async().await;
        });
    }

    /// Toggle infinite-play: flips the persisted autoplay mode between
    /// `InfiniteRadio` and `ContinueWithinSource`, mirroring the Tauri
    /// `handleToggleInfinitePlay` (which calls `setAutoplayMode`).
    pub fn toggle_infinite_play(&self) {
        let this = self.clone();
        self.handle.spawn(async move {
            let enabled = this
                .playback
                .get_preferences()
                .map(|p| p.autoplay_mode == AutoplayMode::InfiniteRadio)
                .unwrap_or(false);
            let next = if enabled {
                AutoplayMode::ContinueWithinSource
            } else {
                AutoplayMode::InfiniteRadio
            };
            if let Err(e) = this.playback.set_autoplay_mode(next) {
                log::error!("[qbz-slint] queue: set autoplay mode failed: {e}");
            }
            this.refresh_async().await;
        });
    }

}
