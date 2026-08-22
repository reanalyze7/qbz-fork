use crate::queue::{InternalState, QueueManager};

impl QueueManager {
    /// Number of upcoming tracks (those after the current one) in the current
    /// play order. Mirrors `get_state_full`'s upcoming computation: shuffle-aware
    /// when shuffle is on, otherwise the tail of `tracks` after `current_index`.
    fn upcoming_len(state: &InternalState) -> usize {
        match state.current_index {
            Some(curr) => {
                if state.shuffle {
                    state
                        .shuffle_order
                        .len()
                        .saturating_sub(state.shuffle_position + 1)
                } else {
                    state.tracks.len().saturating_sub(curr + 1)
                }
            }
            None => state.tracks.len(),
        }
    }

    /// Remove every upcoming track positioned AFTER `upcoming_index` in the
    /// current play order; the track at `upcoming_index` is kept. Works in
    /// UPCOMING space (not absolute `tracks` indices), so it stays correct under
    /// shuffle by reusing `remove_upcoming_track`, which resolves upcoming
    /// positions through `shuffle_order`. Peels positions off the tail inward so
    /// the surviving positions never shift under it. Returns the count removed.
    ///
    /// This is the wired "Remove all after" queue action. (`remove_after`, below,
    /// truncates by absolute `tracks` index and is NOT play-order-aware under
    /// shuffle — it is kept only for its existing unit coverage.)
    pub fn remove_upcoming_after(&self, upcoming_index: usize) -> usize {
        let mut upcoming_len = {
            let state = self.state.lock().unwrap();
            Self::upcoming_len(&state)
        };
        if upcoming_index + 1 >= upcoming_len {
            return 0;
        }
        let mut removed = 0usize;
        while upcoming_len > upcoming_index + 1 {
            if self.remove_upcoming_track(upcoming_len - 1).is_none() {
                break;
            }
            removed += 1;
            upcoming_len -= 1;
        }
        removed
    }

    /// Remove all tracks at indices greater than `index`. The track at
    /// `index` is preserved. Returns the number of tracks removed.
    /// If the marker referenced a track in the removed range, the marker
    /// is cleared. No-op (returns 0) if `index` is the last position or
    /// out of bounds.
    pub fn remove_after(&self, index: usize) -> usize {
        let mut state = self.state.lock().unwrap();

        if index + 1 >= state.tracks.len() {
            return 0;
        }

        let cutoff = index + 1;
        let removed_ids: Vec<u64> = state.tracks[cutoff..].iter().map(|t| t.id).collect();
        let removed_count = removed_ids.len();

        // Drop the tail of `tracks`.
        state.tracks.truncate(cutoff);

        // If shuffle is active, also drop indices >= cutoff from shuffle_order
        // (preserve relative order of surviving indices).
        if state.shuffle {
            state.shuffle_order.retain(|&i| i < cutoff);
            // shuffle_position remains valid since we only dropped tracks AFTER
            // the current playing one (precondition: index >= current_index in
            // the typical UI flow; defensive clamp below handles edge cases).
            if state.shuffle_position >= state.shuffle_order.len() {
                state.shuffle_position = state.shuffle_order.len().saturating_sub(1);
            }
        }

        // Drop history entries pointing past the cutoff.
        state.history.retain(|&i| i < cutoff);

        // Invalidate marker if it pointed into the removed range.
        if let Some(marker_id) = state.stop_after_track_id {
            if removed_ids.contains(&marker_id) {
                state.stop_after_track_id = None;
            }
        }

        removed_count
    }
}
