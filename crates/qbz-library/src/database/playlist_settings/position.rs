//! Manual ordering of playlists.

use rusqlite::params;

use crate::database::PlaylistSettings;
use crate::LibraryError;

use super::super::LibraryDatabase;

impl LibraryDatabase {
    /// Update position for a playlist
    pub fn set_playlist_position(
        &self,
        qobuz_playlist_id: u64,
        position: i32,
    ) -> Result<(), LibraryError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        // First check if settings exist, if not create default
        let existing = self.get_playlist_settings(qobuz_playlist_id)?;
        if existing.is_none() {
            let mut settings = PlaylistSettings::default();
            settings.qobuz_playlist_id = qobuz_playlist_id;
            settings.position = position;
            return self.save_playlist_settings(&settings);
        }

        self.conn
            .execute(
                "UPDATE playlist_settings SET position = ?1, updated_at = ?2
             WHERE qobuz_playlist_id = ?3",
                params![position, now, qobuz_playlist_id as i64],
            )
            .map_err(|e| {
                LibraryError::Database(format!("Failed to update playlist position: {}", e))
            })?;

        Ok(())
    }

    /// Bulk reorder playlists by setting positions
    pub fn reorder_playlists(&self, playlist_ids: &[u64]) -> Result<(), LibraryError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        for (index, &playlist_id) in playlist_ids.iter().enumerate() {
            // Ensure settings exist first
            let existing = self.get_playlist_settings(playlist_id)?;
            if existing.is_none() {
                let mut settings = PlaylistSettings::default();
                settings.qobuz_playlist_id = playlist_id;
                settings.position = index as i32;
                self.save_playlist_settings(&settings)?;
            } else {
                self.conn
                    .execute(
                        "UPDATE playlist_settings SET position = ?1, updated_at = ?2
                     WHERE qobuz_playlist_id = ?3",
                        params![index as i32, now, playlist_id as i64],
                    )
                    .map_err(|e| {
                        LibraryError::Database(format!("Failed to reorder playlists: {}", e))
                    })?;
            }
        }

        Ok(())
    }
}
