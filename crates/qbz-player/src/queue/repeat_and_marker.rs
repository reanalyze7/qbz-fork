use qbz_models::RepeatMode;

use crate::queue::QueueManager;

impl QueueManager {
    /// Set repeat mode
    pub fn set_repeat(&self, mode: RepeatMode) {
        self.state.lock().unwrap().repeat = mode;
    }

    /// Get repeat mode
    pub fn get_repeat(&self) -> RepeatMode {
        self.state.lock().unwrap().repeat
    }

    /// Set the "stop after" marker on a specific track ID. Replaces any
    /// previous marker. Silent no-op if the track ID is not currently in
    /// the queue (defensive check — frontend should only ever pass IDs
    /// from the current queue).
    pub fn set_stop_after(&self, track_id: u64) {
        let mut state = self.state.lock().unwrap();
        if state.tracks.iter().any(|t| t.id == track_id) {
            state.stop_after_track_id = Some(track_id);
        }
    }

    /// Clear the marker (user cancellation from UI).
    pub fn clear_stop_after(&self) {
        let mut state = self.state.lock().unwrap();
        state.stop_after_track_id = None;
    }

    /// Read current marker (used by `get_state()` for serialization).
    pub fn get_stop_after(&self) -> Option<u64> {
        self.state.lock().unwrap().stop_after_track_id
    }

    /// One-shot consume: if the finished track ID matches the marker,
    /// clear it and return true. Otherwise return false. The
    /// auto-advance driver calls this on every natural track-end and
    /// pauses (instead of advancing) when it returns true. Manual skip
    /// paths must NOT call this.
    pub fn consume_stop_after_if(&self, finished_track_id: u64) -> bool {
        let mut state = self.state.lock().unwrap();
        if state.stop_after_track_id == Some(finished_track_id) {
            state.stop_after_track_id = None;
            true
        } else {
            false
        }
    }
}
