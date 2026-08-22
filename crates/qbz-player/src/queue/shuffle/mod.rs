mod internal;

use crate::queue::QueueManager;

impl QueueManager {
    /// Toggle shuffle mode
    pub fn set_shuffle(&self, enabled: bool) {
        let mut state = self.state.lock().unwrap();
        if state.shuffle == enabled {
            return;
        }
        state.shuffle = enabled;

        if enabled {
            Self::regenerate_shuffle_order_internal(&mut state);

            // Enabling shuffle during active playback must keep current track
            // as the first item in the shuffled timeline. Otherwise, indices
            // before current are interpreted as already played.
            if let Some(curr_idx) = state.current_index {
                if let Some(pos) = state.shuffle_order.iter().position(|&idx| idx == curr_idx) {
                    if pos != 0 {
                        state.shuffle_order.swap(0, pos);
                    }
                    state.shuffle_position = 0;
                }
            }
        }
    }

    /// Set shuffle mode using an authoritative order produced elsewhere.
    /// Used by QConnect so the local queue follows the remote session order
    /// instead of generating a second independent shuffle.
    pub fn set_shuffle_with_order(&self, enabled: bool, shuffle_order: Option<Vec<usize>>) {
        let mut state = self.state.lock().unwrap();
        state.shuffle = enabled;

        if !enabled {
            state.shuffle_order.clear();
            state.shuffle_position = 0;
            return;
        }

        if let Some(order) =
            shuffle_order.filter(|order| Self::is_valid_shuffle_order(order, state.tracks.len()))
        {
            state.shuffle_order = order;
            if let Some(curr_idx) = state.current_index {
                if let Some(pos) = state.shuffle_order.iter().position(|&idx| idx == curr_idx) {
                    state.shuffle_position = pos;
                } else {
                    state.shuffle_position = 0;
                }
            } else {
                state.shuffle_position = 0;
            }
            return;
        }

        Self::set_identity_shuffle_order_internal(&mut state);
    }

    /// Get shuffle status
    pub fn is_shuffle(&self) -> bool {
        self.state.lock().unwrap().shuffle
    }
}
