//! Queue position navigation: jump/advance, peek, and the "stop after
//! this song" marker.

use qbz_models::{CoreEvent, FrontendAdapter, QueueTrack};

use super::super::QbzCore;

impl<A: FrontendAdapter + Send + Sync + 'static> QbzCore<A> {
    /// Jump to a specific track by index
    pub async fn play_index(&self, index: usize) -> Option<QueueTrack> {
        let queue = self.queue.write().await;
        let track = queue.play_index(index);
        self.emit(CoreEvent::QueueUpdated {
            state: queue.get_state(),
        })
        .await;
        track
    }

    /// Jump to a track by its position in the upcoming list (as shown in the
    /// Queue sidebar). Shuffle-aware: resolves through `shuffle_order` when
    /// shuffle is active.
    pub async fn play_upcoming_at(&self, upcoming_index: usize) -> Option<QueueTrack> {
        let queue = self.queue.write().await;
        let track = queue.play_upcoming_at(upcoming_index);
        self.emit(CoreEvent::QueueUpdated {
            state: queue.get_state(),
        })
        .await;
        track
    }

    /// Advance to next track in queue
    pub async fn next_track(&self) -> Option<QueueTrack> {
        let queue = self.queue.write().await;
        let track = queue.next();
        self.emit(CoreEvent::QueueUpdated {
            state: queue.get_state(),
        })
        .await;
        track
    }

    /// Go to previous track in queue
    pub async fn previous_track(&self) -> Option<QueueTrack> {
        let queue = self.queue.write().await;
        let track = queue.previous();
        self.emit(CoreEvent::QueueUpdated {
            state: queue.get_state(),
        })
        .await;
        track
    }

    /// Get multiple upcoming tracks without advancing (for prefetching)
    pub async fn peek_upcoming(&self, count: usize) -> Vec<QueueTrack> {
        let queue = self.queue.read().await;
        queue.peek_upcoming(count)
    }

    /// The current queue track, if any (source-aware playback routing).
    pub async fn current_track(&self) -> Option<QueueTrack> {
        let queue = self.queue.read().await;
        queue.current()
    }

    /// Set the "stop after this song" marker on a queue track id. Replaces any
    /// previous marker (single marker). Silent no-op if the id is not in the queue.
    /// Intentionally does NOT emit `CoreEvent::QueueUpdated` — the marker is a UI
    /// intent the frontend reflects via its own queue snapshot, and emitting here
    /// risks QConnect echo loops.
    pub async fn set_stop_after(&self, track_id: u64) {
        self.queue.write().await.set_stop_after(track_id);
    }

    /// Clear the "stop after" marker (user cancellation).
    pub async fn clear_stop_after(&self) {
        self.queue.write().await.clear_stop_after();
    }

    /// Read the current "stop after" marker, if any.
    pub async fn get_stop_after(&self) -> Option<u64> {
        self.queue.read().await.get_stop_after()
    }

    /// One-shot consume: if `finished_track_id` matches the marker, clear it and
    /// return true (the auto-advance driver then halts instead of advancing).
    /// Only the natural end-of-track path may call this — never a manual skip.
    pub async fn consume_stop_after_if(&self, finished_track_id: u64) -> bool {
        self.queue.write().await.consume_stop_after_if(finished_track_id)
    }

    /// Reconcile the queue pointer to the track the audio engine is actually
    /// playing. A gapless hand-off advances inside the player without going
    /// through `next_track`, so the core pointer can lag the live track and
    /// the now-playing card goes stale. This moves the pointer to the track
    /// with `id` and returns it plus whether the pointer moved; a queue
    /// update is emitted only when it did. Frontend-agnostic — the playback
    /// poll loop calls this to keep now-playing in sync (ADR-006).
    pub async fn sync_current_to_id(&self, id: u64) -> Option<(QueueTrack, bool)> {
        let queue = self.queue.write().await;
        let result = queue.sync_current_to_id(id);
        if matches!(result, Some((_, true))) {
            self.emit(CoreEvent::QueueUpdated {
                state: queue.get_state(),
            })
            .await;
        }
        result
    }
}
