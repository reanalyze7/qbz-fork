use crate::queue::{QueueManager, QueueMoveDirection};

impl QueueManager {
    /// Move a track from one position to another
    pub fn move_track(&self, from_index: usize, to_index: usize) -> bool {
        let mut state = self.state.lock().unwrap();

        if state.shuffle {
            // In shuffle mode, DnD indices come from the visible upcoming list,
            // so they must be applied to shuffle_order positions (not absolute
            // track indices in state.tracks).
            let base_pos = state
                .current_index
                .map(|_| state.shuffle_position + 1)
                .unwrap_or(0);
            let from_pos = base_pos + from_index;
            let to_pos = base_pos + to_index;

            if from_pos >= state.shuffle_order.len() || to_pos >= state.shuffle_order.len() {
                return false;
            }

            if from_pos == to_pos {
                return true;
            }

            let moved = state.shuffle_order.remove(from_pos);
            state.shuffle_order.insert(to_pos, moved);

            if let Some(curr_idx) = state.current_index {
                if let Some(pos) = state.shuffle_order.iter().position(|&x| x == curr_idx) {
                    state.shuffle_position = pos;
                }
            } else {
                state.shuffle_position = 0;
            }

            return true;
        }

        let direction: QueueMoveDirection = if from_index > to_index {
            QueueMoveDirection::Up
        } else {
            QueueMoveDirection::Down
        };

        let mut from_idx = from_index;
        let mut to_idx = to_index;

        if let Some(curr_idx) = state.current_index {
            from_idx = from_idx + curr_idx + 1;
            to_idx = to_idx + curr_idx + 1;
        }

        if direction == QueueMoveDirection::Down {
            to_idx = to_idx - 1;
        }

        log::info!(
            "Queue: move_track - {:?} from {} to {} (internal indices:{} -> {}). Tracks in queue: {}",
            direction,
            from_index,
            to_index,
            from_idx,
            to_idx,
            state.tracks.len()
        );

        if from_idx == to_idx {
            return true;
        }

        if from_idx >= state.tracks.len() || to_idx >= state.tracks.len() {
            return false;
        }

        let track = state.tracks.remove(from_idx);
        state.tracks.insert(to_idx, track);

        if let Some(curr_idx) = state.current_index {
            if from_idx == curr_idx {
                state.current_index = Some(to_idx);
            } else if from_idx < curr_idx && to_idx >= curr_idx {
                state.current_index = Some(curr_idx - 1);
            } else if from_idx > curr_idx && to_idx <= curr_idx {
                state.current_index = Some(curr_idx + 1);
            }
        }

        // Keep history aligned after reorder.
        for hist_idx in state.history.iter_mut() {
            *hist_idx = Self::remap_index_after_move(*hist_idx, from_idx, to_idx);
        }

        true
    }

    /// Remap an index after remove+insert move operation.
    fn remap_index_after_move(idx: usize, from_idx: usize, to_idx: usize) -> usize {
        if idx == from_idx {
            return to_idx;
        }

        if from_idx < to_idx {
            // Moved down: [from+1 ..= to] shift left
            if idx > from_idx && idx <= to_idx {
                idx - 1
            } else {
                idx
            }
        } else {
            // Moved up: [to .. from-1] shift right
            if idx >= to_idx && idx < from_idx {
                idx + 1
            } else {
                idx
            }
        }
    }
}
