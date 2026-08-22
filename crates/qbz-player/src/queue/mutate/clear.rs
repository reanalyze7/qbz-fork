use crate::queue::QueueManager;

impl QueueManager {
    /// Clear the queue.
    ///
    /// When `keep_current` is true (default / historical behavior), the track
    /// at `current_index` is preserved as the sole remaining entry so the
    /// "now playing" slot doesn't go dark mid-song. Callers that know nothing
    /// is playing (or want to fully reset) can pass `false` to wipe everything,
    /// including the current track.
    pub fn clear(&self, keep_current: bool) {
        let mut state = self.state.lock().unwrap();
        state.stop_after_track_id = None;

        if keep_current {
            // Keep the track at `current_index`, not always `tracks[0]`.
            // `truncate(1)` was wrong mid-queue: clear while playing track N
            // would leave the first row as now-playing while audio kept N.
            if let Some(idx) = state.current_index {
                if idx < state.tracks.len() {
                    let kept = state.tracks[idx].clone();
                    // History stores indices into `tracks`. Remap by track id
                    // so entries for removed rows drop and any entry that still
                    // refers to the kept track points at index 0.
                    Self::remap_history_by_track_id_internal(&mut state, std::slice::from_ref(&kept));
                    state.tracks = vec![kept];
                    state.current_index = Some(0);
                } else {
                    state.tracks.clear();
                    state.current_index = None;
                    state.history.clear();
                }
            } else {
                state.tracks.clear();
                state.current_index = None;
                state.history.clear();
            }
        } else {
            state.tracks.clear();
            state.current_index = None;
            // Indices into an empty list are meaningless.
            state.history.clear();
        }

        state.shuffle_order.clear();
        state.shuffle_position = 0;
        // Playback history is remapped (or cleared) above. "Clear queue" only
        // affects current/upcoming queue rows, not an intentional history wipe
        // when the kept track still resolves — but removed tracks cannot stay
        // in index-based history.
    }
}
