use qbz_models::QueueTrack;

use crate::queue::QueueManager;

impl QueueManager {
    /// Move the current pointer to the track whose id matches `id`, WITHOUT
    /// starting playback. Used to reconcile the queue pointer to a track the
    /// audio engine already advanced to on its own (a gapless hand-off
    /// happens inside the player, not through `next`), so the now-playing
    /// card never goes stale while the seek bar keeps moving.
    ///
    /// Returns the matched track plus whether the pointer actually moved
    /// (`false` = it was already current). Returns `None` when no queue track
    /// has that id, leaving the pointer untouched.
    pub fn sync_current_to_id(&self, id: u64) -> Option<(QueueTrack, bool)> {
        let mut state = self.state.lock().unwrap();
        let target = state.tracks.iter().position(|t| t.id == id)?;
        let moved = state.current_index != Some(target);
        if moved {
            // Record the outgoing track so `previous` still walks back.
            if let Some(curr_idx) = state.current_index {
                state.history.push_back(curr_idx);
                while state.history.len() > 50 {
                    state.history.pop_front();
                }
            }
            state.current_index = Some(target);
            // Keep the shuffle cursor aligned with the new position.
            if state.shuffle {
                if let Some(pos) = state.shuffle_order.iter().position(|&x| x == target) {
                    state.shuffle_position = pos;
                }
            }
        }
        state.tracks.get(target).cloned().map(|t| (t, moved))
    }

    /// Jump to a track by its position in the `upcoming` list as returned by
    /// `get_state`. This is the position the user sees in the Queue sidebar;
    /// the method resolves it to the correct canonical index even when
    /// shuffle is active (where the display order differs from the canonical
    /// `tracks` order).
    ///
    /// Used by the "click a track in the queue panel" path — fixes issue
    /// #327 where shuffle mode caused a different track than the one
    /// clicked to be played.
    pub fn play_upcoming_at(&self, upcoming_index: usize) -> Option<QueueTrack> {
        let canonical_index = {
            let state = self.state.lock().unwrap();
            match state.current_index {
                Some(_) if state.shuffle => state
                    .shuffle_order
                    .get(state.shuffle_position + 1 + upcoming_index)
                    .copied(),
                Some(curr_idx) => Some(curr_idx + 1 + upcoming_index),
                None => Some(upcoming_index),
            }
        };
        canonical_index.and_then(|idx| self.play_index(idx))
    }

    /// Jump to a specific track by index
    pub fn play_index(&self, index: usize) -> Option<QueueTrack> {
        let mut state = self.state.lock().unwrap();
        if index >= state.tracks.len() {
            return None;
        }

        // Save current to history — ONLY when actually moving to a DIFFERENT
        // track. Jumping to the index already current (e.g. the QConnect
        // controller's `materialize_remote_queue` re-aligning the cursor to the
        // same index via `play_index`, since the stopped local player makes the
        // alignment fire unconditionally) must NOT record a spurious "previous"
        // entry, or the current track shows up duplicated in the History tab.
        // Matches `sync_current_to_id`'s `moved` guard.
        if let Some(curr_idx) = state.current_index {
            if curr_idx != index {
                state.history.push_back(curr_idx);
                while state.history.len() > 50 {
                    state.history.pop_front();
                }
            }
        }

        state.current_index = Some(index);

        if state.shuffle {
            if let Some(pos) = state.shuffle_order.iter().position(|&x| x == index) {
                state.shuffle_position = pos;
            }
        }

        state.tracks.get(index).cloned()
    }
}
