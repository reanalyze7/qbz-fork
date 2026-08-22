//! Small read/clear accessors over the ephemeral session state.

use super::EphemeralLibraryState;
use crate::LocalTrack;

impl EphemeralLibraryState {
    pub fn clear(&self) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.reset();
        }
    }

    /// Resolve a synthetic high id to the cached `LocalTrack`. Returns
    /// `None` if the id is unknown (stale queue entry from a previous
    /// session, race against `clear`, etc.).
    pub fn get_track(&self, id: i64) -> Option<LocalTrack> {
        let inner = self.inner.lock().ok()?;
        inner.tracks.get(&id).cloned()
    }

    /// Snapshot of every track in the current session, in stable id order
    /// (insertion order = scan order). Used by the Slint UI to build a
    /// queue from the whole folder or a single album group.
    pub fn tracks_snapshot(&self) -> Vec<LocalTrack> {
        let Ok(inner) = self.inner.lock() else {
            return Vec::new();
        };
        let mut tracks: Vec<LocalTrack> = inner.tracks.values().cloned().collect();
        tracks.sort_by_key(|t| t.id);
        tracks
    }

    /// The path of the currently-open ephemeral folder, if any.
    pub fn current_folder_path(&self) -> Option<String> {
        self.inner.lock().ok()?.current_folder_path.clone()
    }
}
