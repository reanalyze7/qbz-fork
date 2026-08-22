//! In-flight-fetch tracking and per-track "recently failed" back-off
//! (issue #637): lets the prefetch scheduler avoid re-hammering a track
//! that is currently un-fetchable.

use std::time::Duration;

use super::AudioCache;

impl AudioCache {
    /// Check if a track is currently being fetched
    pub fn is_fetching(&self, track_id: u64) -> bool {
        self.state.lock().unwrap().fetching.contains(&track_id)
    }

    /// Mark a track as being fetched
    pub fn mark_fetching(&self, track_id: u64) {
        self.state.lock().unwrap().fetching.insert(track_id);
    }

    /// Unmark a track as being fetched
    pub fn unmark_fetching(&self, track_id: u64) {
        self.state.lock().unwrap().fetching.remove(&track_id);
    }

    /// Record that a prefetch for this track failed (starts a back-off window).
    pub fn mark_failed(&self, track_id: u64) {
        self.state
            .lock()
            .unwrap()
            .failed
            .insert(track_id, std::time::Instant::now());
    }

    /// True if the track failed to prefetch within `cooldown` — the scheduler
    /// uses this to skip re-hammering a currently un-fetchable track (issue
    /// #637). Expired entries are cleaned up on read.
    pub fn recently_failed(&self, track_id: u64, cooldown: Duration) -> bool {
        let mut state = self.state.lock().unwrap();
        match state.failed.get(&track_id) {
            Some(when) if when.elapsed() < cooldown => true,
            Some(_) => {
                state.failed.remove(&track_id);
                false
            }
            None => false,
        }
    }

    /// Clear a track's failure marker (e.g. once it is successfully cached).
    pub fn clear_failed(&self, track_id: u64) {
        self.state.lock().unwrap().failed.remove(&track_id);
    }
}
