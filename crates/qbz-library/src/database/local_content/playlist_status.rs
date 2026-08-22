//! Track and query the `has_local_content` status of playlists (none /
//! some local / all local), used by the "Offline" playlist filter.

use rusqlite::params;

use crate::database::LocalContentStatus;
use crate::database::PlaylistSettings;
use crate::LibraryError;

use super::super::LibraryDatabase;

impl LibraryDatabase {
    /// Update the has_local_content status for a playlist
    pub fn update_playlist_local_content_status(
        &self,
        qobuz_playlist_id: u64,
        status: LocalContentStatus,
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
            settings.has_local_content = status;
            return self.save_playlist_settings(&settings);
        }

        self.conn
            .execute(
                "UPDATE playlist_settings SET has_local_content = ?1, updated_at = ?2
             WHERE qobuz_playlist_id = ?3",
                params![status.as_str(), now, qobuz_playlist_id as i64],
            )
            .map_err(|e| {
                LibraryError::Database(format!(
                    "Failed to update playlist local content status: {}",
                    e
                ))
            })?;

        Ok(())
    }

    /// Get playlists filtered by local content status
    pub fn get_playlists_by_local_content(
        &self,
        include_partial: bool,
    ) -> Result<Vec<PlaylistSettings>, LibraryError> {
        let query = if include_partial {
            "SELECT qobuz_playlist_id, custom_artwork_path, sort_by, sort_order,
                    last_search_query, notes, hidden, position, has_local_content, is_favorite, folder_id, created_at, updated_at
             FROM playlist_settings
             WHERE has_local_content IN ('some_local', 'all_local')
             ORDER BY position ASC, updated_at DESC"
        } else {
            "SELECT qobuz_playlist_id, custom_artwork_path, sort_by, sort_order,
                    last_search_query, notes, hidden, position, has_local_content, is_favorite, folder_id, created_at, updated_at
             FROM playlist_settings
             WHERE has_local_content = 'all_local'
             ORDER BY position ASC, updated_at DESC"
        };

        let mut stmt = self
            .conn
            .prepare(query)
            .map_err(|e| LibraryError::Database(format!("Failed to prepare statement: {}", e)))?;

        let settings = stmt
            .query_map([], |row| {
                Ok(PlaylistSettings {
                    qobuz_playlist_id: row.get::<_, i64>(0)? as u64,
                    custom_artwork_path: row.get(1)?,
                    sort_by: row.get(2)?,
                    sort_order: row.get(3)?,
                    last_search_query: row.get(4)?,
                    notes: row.get(5)?,
                    hidden: row.get::<_, i32>(6)? != 0,
                    position: row.get(7)?,
                    has_local_content: LocalContentStatus::from_str(
                        &row.get::<_, Option<String>>(8)?.unwrap_or_default(),
                    ),
                    is_favorite: row.get::<_, i32>(9).unwrap_or(0) != 0,
                    folder_id: row.get(10)?,
                    created_at: row.get(11)?,
                    updated_at: row.get(12)?,
                })
            })
            .map_err(|e| {
                LibraryError::Database(format!("Failed to query playlists by local content: {}", e))
            })?;

        settings
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| LibraryError::Database(format!("Failed to collect playlists: {}", e)))
    }
}
