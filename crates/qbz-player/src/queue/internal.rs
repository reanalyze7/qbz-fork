use qbz_models::QueueTrack;

use crate::queue::{InternalState, QueueManager};

impl QueueManager {
    /// Remap history entries from `state.tracks` indices to indices into
    /// `new_tracks`, looking up by track id. Entries whose track id is no
    /// longer present in `new_tracks` are dropped. Must be called with the
    /// lock held and BEFORE `state.tracks` is replaced.
    ///
    /// This preserves history across queue version bumps that don't change
    /// track identity (e.g. pure reorder, shuffle toggle, or an authoritative
    /// remote echo of the current local queue). Bug #316.
    pub(crate) fn remap_history_by_track_id_internal(
        state: &mut InternalState,
        new_tracks: &[QueueTrack],
    ) {
        if state.history.is_empty() || new_tracks.is_empty() || state.tracks.is_empty() {
            state.history.clear();
            return;
        }

        // Build lookup: track_id -> new index. If duplicate ids exist (rare),
        // last occurrence wins; history will still resolve to a valid track.
        let mut new_id_to_idx: std::collections::HashMap<u64, usize> =
            std::collections::HashMap::with_capacity(new_tracks.len());
        for (idx, track) in new_tracks.iter().enumerate() {
            new_id_to_idx.insert(track.id, idx);
        }

        let mut remapped: std::collections::VecDeque<usize> =
            std::collections::VecDeque::with_capacity(state.history.len());
        for &old_idx in state.history.iter() {
            let Some(old_track) = state.tracks.get(old_idx) else {
                continue;
            };
            if let Some(&new_idx) = new_id_to_idx.get(&old_track.id) {
                remapped.push_back(new_idx);
            }
        }
        state.history = remapped;
    }
}
