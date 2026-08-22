use qbz_models::QueueTrack;

use crate::queue::QueueManager;

impl QueueManager {
    /// Set the entire queue (replaces existing)
    pub fn set_queue(&self, new_tracks: Vec<QueueTrack>, start_index: Option<usize>) {
        let mut state = self.state.lock().unwrap();
        state.stop_after_track_id = None;
        // Remap history by track id BEFORE replacing tracks so that legitimate
        // plays survive queue version bumps / reorders. Entries whose track is
        // no longer present are dropped. See bug #316.
        Self::remap_history_by_track_id_internal(&mut state, &new_tracks);
        state.tracks = new_tracks;
        state.current_index = start_index;

        // Regenerate shuffle order
        Self::regenerate_shuffle_order_internal(&mut state);

        // CRITICAL FIX: When shuffle is enabled and we have a start_index,
        // ensure the start_index track is at the BEGINNING of shuffle order
        if state.shuffle {
            if let Some(start_idx) = start_index {
                if start_idx < state.tracks.len() {
                    if let Some(pos) = state.shuffle_order.iter().position(|&x| x == start_idx) {
                        state.shuffle_order.swap(0, pos);
                        state.shuffle_position = 0;

                        log::info!(
                            "Queue: Adjusted shuffle order to start with track index {} (was at position {})",
                            start_idx,
                            pos
                        );
                    }
                }
            }
        }
    }

    /// Replace the queue and playback order in a single atomic update.
    /// This avoids emitting an intermediate locally reshuffled state before an
    /// authoritative remote shuffle order has been applied.
    pub fn set_queue_with_order(
        &self,
        new_tracks: Vec<QueueTrack>,
        start_index: Option<usize>,
        shuffle_enabled: bool,
        shuffle_order: Option<Vec<usize>>,
    ) {
        let mut state = self.state.lock().unwrap();
        state.stop_after_track_id = None;
        // Remap history by track id BEFORE replacing tracks so that legitimate
        // plays survive queue version bumps / reorders. Entries whose track is
        // no longer present are dropped. See bug #316.
        Self::remap_history_by_track_id_internal(&mut state, &new_tracks);
        state.tracks = new_tracks;
        state.current_index = start_index;
        state.shuffle = shuffle_enabled;

        if !shuffle_enabled {
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
}
