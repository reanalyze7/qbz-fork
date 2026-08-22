//! Sort preference, custom artwork, and last search query updates.

use rusqlite::params;

use crate::database::PlaylistSettings;
use crate::LibraryError;

use super::super::LibraryDatabase;

impl LibraryDatabase {
    /// Update just the sort settings for a playlist
    pub fn update_playlist_sort(
        &self,
        qobuz_playlist_id: u64,
        sort_by: &str,
        sort_order: &str,
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
            settings.sort_by = sort_by.to_string();
            settings.sort_order = sort_order.to_string();
            return self.save_playlist_settings(&settings);
        }

        self.conn
            .execute(
                "UPDATE playlist_settings SET sort_by = ?1, sort_order = ?2, updated_at = ?3
             WHERE qobuz_playlist_id = ?4",
                params![sort_by, sort_order, now, qobuz_playlist_id as i64],
            )
            .map_err(|e| {
                LibraryError::Database(format!("Failed to update playlist sort: {}", e))
            })?;

        Ok(())
    }

    /// Update custom artwork path for a playlist
    pub fn update_playlist_artwork(
        &self,
        qobuz_playlist_id: u64,
        artwork_path: Option<&str>,
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
            settings.custom_artwork_path = artwork_path.map(|s| s.to_string());
            return self.save_playlist_settings(&settings);
        }

        self.conn
            .execute(
                "UPDATE playlist_settings SET custom_artwork_path = ?1, updated_at = ?2
             WHERE qobuz_playlist_id = ?3",
                params![artwork_path, now, qobuz_playlist_id as i64],
            )
            .map_err(|e| {
                LibraryError::Database(format!("Failed to update playlist artwork: {}", e))
            })?;

        Ok(())
    }

    /// Update last search query for a playlist
    pub fn update_playlist_search_query(
        &self,
        qobuz_playlist_id: u64,
        query: Option<&str>,
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
            settings.last_search_query = query.map(|s| s.to_string());
            return self.save_playlist_settings(&settings);
        }

        self.conn
            .execute(
                "UPDATE playlist_settings SET last_search_query = ?1, updated_at = ?2
             WHERE qobuz_playlist_id = ?3",
                params![query, now, qobuz_playlist_id as i64],
            )
            .map_err(|e| {
                LibraryError::Database(format!("Failed to update playlist search query: {}", e))
            })?;

        Ok(())
    }
}
