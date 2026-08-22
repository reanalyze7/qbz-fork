//! Hidden/favorite flags and copy-tracking for playlists sourced elsewhere.

use rusqlite::params;

use crate::database::PlaylistSettings;
use crate::LibraryError;

use super::super::LibraryDatabase;

impl LibraryDatabase {
    /// Update hidden status for a playlist
    pub fn set_playlist_hidden(
        &self,
        qobuz_playlist_id: u64,
        hidden: bool,
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
            settings.hidden = hidden;
            return self.save_playlist_settings(&settings);
        }

        self.conn
            .execute(
                "UPDATE playlist_settings SET hidden = ?1, updated_at = ?2
             WHERE qobuz_playlist_id = ?3",
                params![hidden as i32, now, qobuz_playlist_id as i64],
            )
            .map_err(|e| {
                LibraryError::Database(format!("Failed to update playlist hidden: {}", e))
            })?;

        Ok(())
    }

    /// Update favorite status for a playlist
    pub fn set_playlist_favorite(
        &self,
        qobuz_playlist_id: u64,
        favorite: bool,
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
            settings.is_favorite = favorite;
            return self.save_playlist_settings(&settings);
        }

        self.conn
            .execute(
                "UPDATE playlist_settings SET is_favorite = ?1, updated_at = ?2
             WHERE qobuz_playlist_id = ?3",
                params![favorite as i32, now, qobuz_playlist_id as i64],
            )
            .map_err(|e| {
                LibraryError::Database(format!("Failed to update playlist favorite: {}", e))
            })?;

        Ok(())
    }

    /// Get all playlist IDs that are marked as favorites
    pub fn get_favorite_playlist_ids(&self) -> Result<Vec<u64>, LibraryError> {
        let mut stmt = self.conn.prepare(
            "SELECT qobuz_playlist_id FROM playlist_settings WHERE is_favorite = 1 ORDER BY updated_at DESC"
        ).map_err(|e| LibraryError::Database(format!("Failed to prepare statement: {}", e)))?;

        let ids = stmt
            .query_map([], |row| Ok(row.get::<_, i64>(0)? as u64))
            .map_err(|e| {
                LibraryError::Database(format!("Failed to query favorite playlists: {}", e))
            })?;

        ids.collect::<Result<Vec<_>, _>>().map_err(|e| {
            LibraryError::Database(format!("Failed to collect favorite playlist IDs: {}", e))
        })
    }

    /// Record that a Qobuz playlist (by its SOURCE id) was copied into the
    /// user's library. Idempotent — re-copying the same source is a no-op.
    pub fn mark_playlist_copied(&self, qobuz_playlist_id: u64) -> Result<(), LibraryError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        self.conn
            .execute(
                "INSERT OR IGNORE INTO copied_playlists (qobuz_playlist_id, copied_at) VALUES (?1, ?2)",
                params![qobuz_playlist_id as i64, now],
            )
            .map_err(|e| {
                LibraryError::Database(format!("Failed to mark playlist copied: {}", e))
            })?;
        Ok(())
    }

    /// Whether a Qobuz playlist (by its SOURCE id) has already been copied into
    /// the user's library — used to hide the Copy button on its detail view.
    pub fn is_playlist_copied(&self, qobuz_playlist_id: u64) -> Result<bool, LibraryError> {
        let count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM copied_playlists WHERE qobuz_playlist_id = ?1",
                params![qobuz_playlist_id as i64],
                |row| row.get(0),
            )
            .map_err(|e| {
                LibraryError::Database(format!("Failed to check copied playlist: {}", e))
            })?;
        Ok(count > 0)
    }
}
