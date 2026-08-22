use crate::queue::{InternalState, QueueManager};

impl QueueManager {
    /// Regenerate shuffle order (internal, must be called with lock held)
    pub(crate) fn regenerate_shuffle_order_internal(state: &mut InternalState) {
        let mut order: Vec<usize> = (0..state.tracks.len()).collect();

        // Fisher-Yates shuffle with proper PRNG
        use rand::{Rng, SeedableRng};
        use std::time::{SystemTime, UNIX_EPOCH};

        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

        for i in (1..order.len()).rev() {
            let j = rng.random_range(0..=i);
            order.swap(i, j);
        }

        state.shuffle_order = order;

        if let Some(curr_idx) = state.current_index {
            if let Some(pos) = state.shuffle_order.iter().position(|&x| x == curr_idx) {
                state.shuffle_position = pos;
            } else {
                state.shuffle_position = 0;
            }
        } else {
            state.shuffle_position = 0;
        }
    }

    /// Preserve the existing queue order when shuffle is remote-controlled but
    /// no authoritative remote order has arrived yet.
    pub(crate) fn set_identity_shuffle_order_internal(state: &mut InternalState) {
        state.shuffle_order = (0..state.tracks.len()).collect();

        if let Some(curr_idx) = state.current_index {
            state.shuffle_position = curr_idx.min(state.shuffle_order.len().saturating_sub(1));
        } else {
            state.shuffle_position = 0;
        }
    }

    /// Remove one absolute track index from shuffle order and rebase remaining indices.
    pub(crate) fn remove_index_from_shuffle_internal(state: &mut InternalState, removed_idx: usize) {
        if let Some(pos) = state
            .shuffle_order
            .iter()
            .position(|&idx| idx == removed_idx)
        {
            state.shuffle_order.remove(pos);

            if pos < state.shuffle_position && state.shuffle_position > 0 {
                state.shuffle_position -= 1;
            } else if pos == state.shuffle_position
                && state.shuffle_position >= state.shuffle_order.len()
            {
                state.shuffle_position = state.shuffle_order.len().saturating_sub(1);
            }
        }

        for idx in state.shuffle_order.iter_mut() {
            if *idx > removed_idx {
                *idx -= 1;
            }
        }

        if let Some(curr_idx) = state.current_index {
            if let Some(pos) = state.shuffle_order.iter().position(|&idx| idx == curr_idx) {
                state.shuffle_position = pos;
            } else {
                state.shuffle_position = 0;
            }
        } else {
            state.shuffle_position = 0;
        }
    }

    pub(crate) fn is_valid_shuffle_order(order: &[usize], track_count: usize) -> bool {
        if order.len() != track_count {
            return false;
        }

        let mut seen = vec![false; track_count];
        for &idx in order {
            if idx >= track_count || seen[idx] {
                return false;
            }
            seen[idx] = true;
        }

        true
    }
}
