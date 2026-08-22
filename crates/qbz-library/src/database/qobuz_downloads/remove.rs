//! Remove Qobuz-downloaded tracks (single or all) from `local_tracks`.

use rusqlite::params;

use crate::LibraryError;

use super::super::LibraryDatabase;

impl LibraryDatabase {
    /// Remove a Qobuz cached track from the library by track_id
    /// Note: Database source field remains 'qobuz_download' for compatibility
    pub fn remove_qobuz_cached_track(&self, qobuz_track_id: u64) -> Result<(), LibraryError> {
        self.conn
            .execute(
                "DELETE FROM local_tracks WHERE qobuz_track_id = ?1 AND source = 'qobuz_download'",
                params![qobuz_track_id as i64],
            )
            .map_err(|e| {
                LibraryError::Database(format!("Failed to remove Qobuz cached track: {}", e))
            })?;
        Ok(())
    }

    /// Remove all Qobuz cached tracks from the library
    /// Note: Database source field remains 'qobuz_download' for compatibility
    pub fn remove_all_qobuz_cached_tracks(&self) -> Result<usize, LibraryError> {
        let count = self
            .conn
            .execute(
                "DELETE FROM local_tracks WHERE source = 'qobuz_download'",
                [],
            )
            .map_err(|e| {
                LibraryError::Database(format!("Failed to remove all Qobuz cached tracks: {}", e))
            })?;
        Ok(count)
    }
}
