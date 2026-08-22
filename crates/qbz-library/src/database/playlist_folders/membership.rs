//! Folder deletion, reordering, and playlist-to-folder membership queries.

use rusqlite::params;

use crate::database::PlaylistSettings;
use crate::LibraryError;

use super::super::LibraryDatabase;

impl LibraryDatabase {
    /// Delete a playlist folder (playlists return to root via ON DELETE SET NULL)
    pub fn delete_playlist_folder(&self, folder_id: &str) -> Result<(), LibraryError> {
        self.conn
            .execute(
                "DELETE FROM playlist_folders WHERE id = ?1",
                params![folder_id],
            )
            .map_err(|e| {
                LibraryError::Database(format!("Failed to delete playlist folder: {}", e))
            })?;

        Ok(())
    }

    /// Reorder playlist folders
    pub fn reorder_playlist_folders(&self, folder_ids: &[String]) -> Result<(), LibraryError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        for (position, folder_id) in folder_ids.iter().enumerate() {
            self.conn
                .execute(
                    "UPDATE playlist_folders SET position = ?1, updated_at = ?2 WHERE id = ?3",
                    params![position as i32, now, folder_id],
                )
                .map_err(|e| LibraryError::Database(format!("Failed to reorder folder: {}", e)))?;
        }

        Ok(())
    }

    /// Move a playlist to a folder (or root if folder_id is None)
    pub fn move_playlist_to_folder(
        &self,
        qobuz_playlist_id: u64,
        folder_id: Option<&str>,
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
            settings.folder_id = folder_id.map(|s| s.to_string());
            return self.save_playlist_settings(&settings);
        }

        self.conn.execute(
            "UPDATE playlist_settings SET folder_id = ?1, updated_at = ?2 WHERE qobuz_playlist_id = ?3",
            params![folder_id, now, qobuz_playlist_id as i64],
        ).map_err(|e| LibraryError::Database(format!("Failed to move playlist to folder: {}", e)))?;

        Ok(())
    }

    /// Get playlists in a specific folder (or root if folder_id is None)
    pub fn get_playlists_in_folder(
        &self,
        folder_id: Option<&str>,
    ) -> Result<Vec<u64>, LibraryError> {
        if let Some(fid) = folder_id {
            let mut stmt = self.conn.prepare(
                "SELECT qobuz_playlist_id FROM playlist_settings WHERE folder_id = ?1 ORDER BY position ASC"
            ).map_err(|e| LibraryError::Database(format!("Failed to prepare statement: {}", e)))?;

            let ids = stmt
                .query_map(params![fid], |row| Ok(row.get::<_, i64>(0)? as u64))
                .map_err(|e| {
                    LibraryError::Database(format!("Failed to query playlists in folder: {}", e))
                })?;

            ids.collect::<Result<Vec<_>, _>>().map_err(|e| {
                LibraryError::Database(format!("Failed to collect playlist IDs: {}", e))
            })
        } else {
            let mut stmt = self.conn.prepare(
                "SELECT qobuz_playlist_id FROM playlist_settings WHERE folder_id IS NULL ORDER BY position ASC"
            ).map_err(|e| LibraryError::Database(format!("Failed to prepare statement: {}", e)))?;

            let ids = stmt
                .query_map([], |row| Ok(row.get::<_, i64>(0)? as u64))
                .map_err(|e| {
                    LibraryError::Database(format!("Failed to query playlists in folder: {}", e))
                })?;

            ids.collect::<Result<Vec<_>, _>>().map_err(|e| {
                LibraryError::Database(format!("Failed to collect playlist IDs: {}", e))
            })
        }
    }
}
