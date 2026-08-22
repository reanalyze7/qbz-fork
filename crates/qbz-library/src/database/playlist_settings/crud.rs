//! Core CRUD operations for playlist settings rows.

use rusqlite::{params, OptionalExtension};

use crate::database::{LocalContentStatus, PlaylistSettings};
use crate::LibraryError;

use super::super::LibraryDatabase;

impl LibraryDatabase {
    // === Playlist Settings ===

    /// Get playlist settings by Qobuz playlist ID
    pub fn get_playlist_settings(
        &self,
        qobuz_playlist_id: u64,
    ) -> Result<Option<PlaylistSettings>, LibraryError> {
        let result = self.conn.query_row(
            "SELECT qobuz_playlist_id, custom_artwork_path, sort_by, sort_order,
                    last_search_query, notes, hidden, position, has_local_content, is_favorite, folder_id, created_at, updated_at
             FROM playlist_settings WHERE qobuz_playlist_id = ?1",
            params![qobuz_playlist_id as i64],
            |row| {
                Ok(PlaylistSettings {
                    qobuz_playlist_id: row.get::<_, i64>(0)? as u64,
                    custom_artwork_path: row.get(1)?,
                    sort_by: row.get(2)?,
                    sort_order: row.get(3)?,
                    last_search_query: row.get(4)?,
                    notes: row.get(5)?,
                    hidden: row.get::<_, i32>(6)? != 0,
                    position: row.get(7)?,
                    has_local_content: LocalContentStatus::from_str(&row.get::<_, Option<String>>(8)?.unwrap_or_default()),
                    is_favorite: row.get::<_, i32>(9).unwrap_or(0) != 0,
                    folder_id: row.get(10)?,
                    created_at: row.get(11)?,
                    updated_at: row.get(12)?,
                })
            },
        ).optional()
        .map_err(|e| LibraryError::Database(format!("Failed to get playlist settings: {}", e)))?;

        Ok(result)
    }

    /// Save or update playlist settings
    pub fn save_playlist_settings(&self, settings: &PlaylistSettings) -> Result<(), LibraryError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        self.conn.execute(
            "INSERT INTO playlist_settings
                (qobuz_playlist_id, custom_artwork_path, sort_by, sort_order,
                 last_search_query, notes, hidden, position, has_local_content, is_favorite, folder_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
             ON CONFLICT(qobuz_playlist_id) DO UPDATE SET
                custom_artwork_path = excluded.custom_artwork_path,
                sort_by = excluded.sort_by,
                sort_order = excluded.sort_order,
                last_search_query = excluded.last_search_query,
                notes = excluded.notes,
                hidden = excluded.hidden,
                position = excluded.position,
                has_local_content = excluded.has_local_content,
                is_favorite = excluded.is_favorite,
                folder_id = excluded.folder_id,
                updated_at = excluded.updated_at",
            params![
                settings.qobuz_playlist_id as i64,
                &settings.custom_artwork_path,
                &settings.sort_by,
                &settings.sort_order,
                &settings.last_search_query,
                &settings.notes,
                settings.hidden as i32,
                settings.position,
                settings.has_local_content.as_str(),
                settings.is_favorite as i32,
                &settings.folder_id,
                settings.created_at,
                now,
            ],
        ).map_err(|e| LibraryError::Database(format!("Failed to save playlist settings: {}", e)))?;

        Ok(())
    }

    /// Delete playlist settings
    pub fn delete_playlist_settings(&self, qobuz_playlist_id: u64) -> Result<(), LibraryError> {
        self.conn
            .execute(
                "DELETE FROM playlist_settings WHERE qobuz_playlist_id = ?1",
                params![qobuz_playlist_id as i64],
            )
            .map_err(|e| {
                LibraryError::Database(format!("Failed to delete playlist settings: {}", e))
            })?;

        Ok(())
    }
}
