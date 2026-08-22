//! Bulk read of every playlist's settings row (syncing/export).

use crate::database::{LocalContentStatus, PlaylistSettings};
use crate::LibraryError;

use super::super::LibraryDatabase;

impl LibraryDatabase {
    /// Get all playlist settings (for syncing/export)
    pub fn get_all_playlist_settings(&self) -> Result<Vec<PlaylistSettings>, LibraryError> {
        let mut stmt = self.conn.prepare(
            "SELECT qobuz_playlist_id, custom_artwork_path, sort_by, sort_order,
                    last_search_query, notes, hidden, position, has_local_content, is_favorite, folder_id, created_at, updated_at
             FROM playlist_settings ORDER BY position ASC, updated_at DESC"
        ).map_err(|e| LibraryError::Database(format!("Failed to prepare statement: {}", e)))?;

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
                LibraryError::Database(format!("Failed to query playlist settings: {}", e))
            })?;

        settings.collect::<Result<Vec<_>, _>>().map_err(|e| {
            LibraryError::Database(format!("Failed to collect playlist settings: {}", e))
        })
    }
}
