use qbz_models::{QueueState, QueueTrack};

use crate::queue::QueueManager;

impl QueueManager {
    /// Get queue state for frontend
    pub fn get_state(&self) -> QueueState {
        let state = self.state.lock().unwrap();

        let current_track = state
            .current_index
            .and_then(|idx| state.tracks.get(idx).cloned());

        // Get upcoming tracks (after current)
        let upcoming: Vec<QueueTrack> = if let Some(curr_idx) = state.current_index {
            if state.shuffle {
                state
                    .shuffle_order
                    .iter()
                    .skip(state.shuffle_position + 1)
                    .take(20)
                    .filter_map(|&idx| state.tracks.get(idx).cloned())
                    .collect()
            } else {
                state
                    .tracks
                    .iter()
                    .skip(curr_idx + 1)
                    .take(20)
                    .cloned()
                    .collect()
            }
        } else {
            state.tracks.iter().take(20).cloned().collect()
        };

        // Get history tracks (recent first)
        let history_tracks: Vec<QueueTrack> = state
            .history
            .iter()
            .rev()
            .take(10)
            .filter_map(|&idx| state.tracks.get(idx).cloned())
            .collect();

        QueueState {
            current_track,
            current_index: state.current_index,
            upcoming,
            history: history_tracks,
            shuffle: state.shuffle,
            repeat: state.repeat,
            total_tracks: state.tracks.len(),
            stop_after_track_id: state.stop_after_track_id,
        }
    }

    /// Get all tracks in the queue plus the current index (for session persistence).
    /// Unlike get_state() which caps upcoming/history, this returns the full track list.
    pub fn get_all_tracks(&self) -> (Vec<QueueTrack>, Option<usize>) {
        let state = self.state.lock().unwrap();
        (state.tracks.clone(), state.current_index)
    }

    /// Get the full queue state without the upcoming/history caps applied by
    /// `get_state()`. Used by clients that paginate the upcoming list (e.g.
    /// the Queue sidebar's "UP NEXT" paginator) and need the complete history.
    /// The `upcoming` ordering is shuffle-aware, matching `get_state()`.
    pub fn get_state_full(&self) -> QueueState {
        let state = self.state.lock().unwrap();

        let current_track = state
            .current_index
            .and_then(|idx| state.tracks.get(idx).cloned());

        // Full upcoming list (after current), shuffle-aware. No `take` cap.
        let upcoming: Vec<QueueTrack> = if let Some(curr_idx) = state.current_index {
            if state.shuffle {
                state
                    .shuffle_order
                    .iter()
                    .skip(state.shuffle_position + 1)
                    .filter_map(|&idx| state.tracks.get(idx).cloned())
                    .collect()
            } else {
                state
                    .tracks
                    .iter()
                    .skip(curr_idx + 1)
                    .cloned()
                    .collect()
            }
        } else {
            state.tracks.clone()
        };

        // Full history (recent first). No `take` cap.
        let history_tracks: Vec<QueueTrack> = state
            .history
            .iter()
            .rev()
            .filter_map(|&idx| state.tracks.get(idx).cloned())
            .collect();

        QueueState {
            current_track,
            current_index: state.current_index,
            upcoming,
            history: history_tracks,
            shuffle: state.shuffle,
            repeat: state.repeat,
            total_tracks: state.tracks.len(),
            stop_after_track_id: state.stop_after_track_id,
        }
    }
}
