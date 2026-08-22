use qbz_models::QueueTrack;

use crate::queue::QueueManager;

impl QueueManager {
    /// Remove a track by index
    pub fn remove_track(&self, index: usize) -> Option<QueueTrack> {
        let mut state = self.state.lock().unwrap();
        if index >= state.tracks.len() {
            return None;
        }

        let removed = state.tracks.remove(index);

        // Invalidate marker if the removed track matches
        if state.stop_after_track_id == Some(removed.id) {
            state.stop_after_track_id = None;
        }

        // Adjust current index if needed
        if let Some(curr_idx) = state.current_index {
            if index < curr_idx {
                state.current_index = Some(curr_idx - 1);
            } else if index == curr_idx {
                if curr_idx >= state.tracks.len() {
                    state.current_index = if state.tracks.is_empty() {
                        None
                    } else {
                        Some(state.tracks.len() - 1)
                    };
                }
            }
        }

        // Keep history indices aligned with current track list after removal.
        state.history.retain(|&hist_idx| hist_idx != index);
        for hist_idx in state.history.iter_mut() {
            if *hist_idx > index {
                *hist_idx -= 1;
            }
        }

        if state.shuffle {
            Self::remove_index_from_shuffle_internal(&mut state, index);
        }
        Some(removed)
    }

    /// Remove a track by its position in the upcoming list
    pub fn remove_upcoming_track(&self, upcoming_index: usize) -> Option<QueueTrack> {
        let mut state = self.state.lock().unwrap();

        let actual_index = if state.shuffle {
            let shuffle_pos = state.shuffle_position + 1 + upcoming_index;
            if shuffle_pos >= state.shuffle_order.len() {
                return None;
            }
            state.shuffle_order[shuffle_pos]
        } else {
            match state.current_index {
                Some(curr_idx) => curr_idx + 1 + upcoming_index,
                None => upcoming_index,
            }
        };

        if actual_index >= state.tracks.len() {
            return None;
        }

        log::info!(
            "remove_upcoming_track: upcoming_index={} -> actual_index={}",
            upcoming_index,
            actual_index
        );

        let removed = state.tracks.remove(actual_index);

        // Invalidate marker if the removed track matches
        if state.stop_after_track_id == Some(removed.id) {
            state.stop_after_track_id = None;
        }

        if let Some(curr_idx) = state.current_index {
            if actual_index < curr_idx {
                state.current_index = Some(curr_idx - 1);
            } else if actual_index == curr_idx {
                if curr_idx >= state.tracks.len() {
                    state.current_index = if state.tracks.is_empty() {
                        None
                    } else {
                        Some(state.tracks.len() - 1)
                    };
                }
            }
        }

        // Keep history indices aligned with current track list after removal.
        state.history.retain(|&hist_idx| hist_idx != actual_index);
        for hist_idx in state.history.iter_mut() {
            if *hist_idx > actual_index {
                *hist_idx -= 1;
            }
        }

        if state.shuffle {
            Self::remove_index_from_shuffle_internal(&mut state, actual_index);
        }
        Some(removed)
    }
}
