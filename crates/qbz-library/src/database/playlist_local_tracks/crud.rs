use rusqlite::params;

use crate::LibraryError;

use super::super::LibraryDatabase;

impl LibraryDatabase {
    /// Add a local track to a playlist
    pub fn add_local_track_to_playlist(
        &self,
        qobuz_playlist_id: u64,
        local_track_id: i64,
        position: i32,
    ) -> Result<(), LibraryError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        self.conn
            .execute(
                "INSERT OR REPLACE INTO playlist_local_tracks
                (qobuz_playlist_id, local_track_id, position, added_at)
             VALUES (?1, ?2, ?3, ?4)",
                params![qobuz_playlist_id as i64, local_track_id, position, now],
            )
            .map_err(|e| {
                LibraryError::Database(format!("Failed to add local track to playlist: {}", e))
            })?;

        Ok(())
    }

    /// Remove a local track from a playlist
    pub fn remove_local_track_from_playlist(
        &self,
        qobuz_playlist_id: u64,
        local_track_id: i64,
    ) -> Result<(), LibraryError> {
        self.conn
            .execute(
                "DELETE FROM playlist_local_tracks
             WHERE qobuz_playlist_id = ?1 AND local_track_id = ?2",
                params![qobuz_playlist_id as i64, local_track_id],
            )
            .map_err(|e| {
                LibraryError::Database(format!("Failed to remove local track from playlist: {}", e))
            })?;

        Ok(())
    }

    /// Update position of a local track in a playlist
    pub fn update_local_track_position(
        &self,
        qobuz_playlist_id: u64,
        local_track_id: i64,
        new_position: i32,
    ) -> Result<(), LibraryError> {
        self.conn
            .execute(
                "UPDATE playlist_local_tracks SET position = ?1
             WHERE qobuz_playlist_id = ?2 AND local_track_id = ?3",
                params![new_position, qobuz_playlist_id as i64, local_track_id],
            )
            .map_err(|e| {
                LibraryError::Database(format!("Failed to update local track position: {}", e))
            })?;

        Ok(())
    }

    /// Clear all local tracks from a playlist
    pub fn clear_playlist_local_tracks(&self, qobuz_playlist_id: u64) -> Result<(), LibraryError> {
        self.conn
            .execute(
                "DELETE FROM playlist_local_tracks WHERE qobuz_playlist_id = ?1",
                params![qobuz_playlist_id as i64],
            )
            .map_err(|e| {
                LibraryError::Database(format!("Failed to clear playlist local tracks: {}", e))
            })?;

        Ok(())
    }
}
