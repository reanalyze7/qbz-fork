use qbz_models::{QueueTrack, RepeatMode};

use crate::queue::QueueManager;

impl QueueManager {
    /// Advance to next track and return it
    pub fn next(&self) -> Option<QueueTrack> {
        let mut state = self.state.lock().unwrap();
        if state.tracks.is_empty() {
            return None;
        }

        // Save current to history before moving
        if let Some(curr_idx) = state.current_index {
            state.history.push_back(curr_idx);
            while state.history.len() > 50 {
                state.history.pop_front();
            }
        }

        if state.repeat == RepeatMode::One {
            return state
                .current_index
                .and_then(|idx| state.tracks.get(idx).cloned());
        }

        let next_idx = if state.shuffle {
            state.shuffle_position += 1;
            if state.shuffle_position < state.shuffle_order.len() {
                Some(state.shuffle_order[state.shuffle_position])
            } else if state.repeat == RepeatMode::All {
                state.shuffle_position = 0;
                state.shuffle_order.first().copied()
            } else {
                None
            }
        } else {
            let curr_idx = state.current_index.unwrap_or(0);
            let next_idx = curr_idx + 1;
            if next_idx < state.tracks.len() {
                Some(next_idx)
            } else if state.repeat == RepeatMode::All {
                Some(0)
            } else {
                None
            }
        };

        state.current_index = next_idx;
        next_idx.and_then(|idx| state.tracks.get(idx).cloned())
    }

    /// Go to previous track and return it
    pub fn previous(&self) -> Option<QueueTrack> {
        let mut state = self.state.lock().unwrap();
        if state.tracks.is_empty() {
            return None;
        }

        // Try to get from history first
        if let Some(prev_idx) = state.history.pop_back() {
            state.current_index = Some(prev_idx);

            if state.shuffle {
                if let Some(pos) = state.shuffle_order.iter().position(|&x| x == prev_idx) {
                    state.shuffle_position = pos;
                }
            }

            return state.tracks.get(prev_idx).cloned();
        }

        // No history, go to previous in order
        let prev_idx = if state.shuffle {
            if state.shuffle_position > 0 {
                state.shuffle_position -= 1;
                Some(state.shuffle_order[state.shuffle_position])
            } else if state.repeat == RepeatMode::All {
                state.shuffle_position = state.shuffle_order.len().saturating_sub(1);
                state.shuffle_order.last().copied()
            } else {
                state.shuffle_order.first().copied()
            }
        } else {
            let curr_idx = state.current_index.unwrap_or(0);
            if curr_idx > 0 {
                Some(curr_idx - 1)
            } else if state.repeat == RepeatMode::All {
                Some(state.tracks.len().saturating_sub(1))
            } else {
                Some(0)
            }
        };

        state.current_index = prev_idx;
        prev_idx.and_then(|idx| state.tracks.get(idx).cloned())
    }
}
