use qbz_models::{QueueTrack, RepeatMode};

use crate::queue::QueueManager;

impl QueueManager {
    /// Get current track
    pub fn current_track(&self) -> Option<QueueTrack> {
        let state = self.state.lock().unwrap();
        state
            .current_index
            .and_then(|idx| state.tracks.get(idx).cloned())
    }

    /// Get next track without advancing
    pub fn peek_next(&self) -> Option<QueueTrack> {
        let state = self.state.lock().unwrap();
        if state.tracks.is_empty() {
            return None;
        }

        if state.repeat == RepeatMode::One {
            return state
                .current_index
                .and_then(|idx| state.tracks.get(idx).cloned());
        }

        let next_idx = if state.shuffle {
            let next_pos = state.shuffle_position + 1;
            if next_pos < state.shuffle_order.len() {
                Some(state.shuffle_order[next_pos])
            } else if state.repeat == RepeatMode::All {
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

        next_idx.and_then(|idx| state.tracks.get(idx).cloned())
    }

    /// Get multiple upcoming tracks without advancing
    pub fn peek_upcoming(&self, count: usize) -> Vec<QueueTrack> {
        let state = self.state.lock().unwrap();
        if state.tracks.is_empty() || count == 0 {
            return Vec::new();
        }

        if state.repeat == RepeatMode::One {
            return Vec::new();
        }

        let mut result = Vec::with_capacity(count);

        if state.shuffle {
            let start_pos = state.shuffle_position + 1;
            for i in 0..count {
                let pos = start_pos + i;
                if pos < state.shuffle_order.len() {
                    if let Some(track) = state.tracks.get(state.shuffle_order[pos]) {
                        result.push(track.clone());
                    }
                } else if state.repeat == RepeatMode::All {
                    let wrapped_pos = pos % state.shuffle_order.len();
                    if let Some(track) = state.tracks.get(state.shuffle_order[wrapped_pos]) {
                        result.push(track.clone());
                    }
                }
            }
        } else {
            let start_idx = state.current_index.map(|i| i + 1).unwrap_or(0);
            for i in 0..count {
                let idx = start_idx + i;
                if idx < state.tracks.len() {
                    result.push(state.tracks[idx].clone());
                } else if state.repeat == RepeatMode::All {
                    let wrapped_idx = idx % state.tracks.len();
                    result.push(state.tracks[wrapped_idx].clone());
                }
            }
        }

        result
    }
}
