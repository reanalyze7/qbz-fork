//! Downloaded-purchases registry: tracks which purchased tracks have been
//! downloaded locally, in which format, so playback/UI can tell a
//! purchased-but-not-downloaded track from one already on disk.

use crate::LibraryError;

use super::LibraryDatabase;

impl LibraryDatabase {
    /// Record a track as downloaded on this computer with its format.
    pub fn mark_purchase_downloaded(
        &self,
        track_id: i64,
        album_id: Option<&str>,
        file_path: &str,
        format_id: i64,
    ) -> Result<(), LibraryError> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO downloaded_purchases (track_id, format_id, album_id, file_path, downloaded_at)
                 VALUES (?1, ?2, ?3, ?4, datetime('now'))",
                rusqlite::params![track_id, format_id, album_id, file_path],
            )
            .map_err(|e| {
                LibraryError::Database(format!("Failed to mark purchase downloaded: {}", e))
            })?;
        Ok(())
    }

    /// Remove a downloaded purchase record (e.g. user deleted the file).
    pub fn remove_downloaded_purchase(&self, track_id: i64) -> Result<(), LibraryError> {
        self.conn
            .execute(
                "DELETE FROM downloaded_purchases WHERE track_id = ?1",
                [track_id],
            )
            .map_err(|e| {
                LibraryError::Database(format!("Failed to remove downloaded purchase: {}", e))
            })?;
        Ok(())
    }

    /// Get all downloaded track IDs for fast lookup (any format).
    /// Automatically removes stale entries where the file no longer exists on disk.
    pub fn get_downloaded_purchase_track_ids(&self) -> Result<Vec<i64>, LibraryError> {
        let mut stmt = self
            .conn
            .prepare("SELECT track_id, format_id, file_path FROM downloaded_purchases")
            .map_err(|e| LibraryError::Database(format!("Failed to prepare statement: {}", e)))?;

        let rows: Vec<(i64, i64, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .map_err(|e| {
                LibraryError::Database(format!("Failed to query downloaded purchases: {}", e))
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| LibraryError::Database(format!("Failed to collect rows: {}", e)))?;

        let mut stale: Vec<(i64, i64)> = Vec::new();
        let mut valid_ids: Vec<i64> = Vec::new();

        for (track_id, format_id, file_path) in &rows {
            if std::path::Path::new(file_path).exists() {
                valid_ids.push(*track_id);
            } else {
                stale.push((*track_id, *format_id));
            }
        }

        // Remove stale entries where the file no longer exists
        if !stale.is_empty() {
            log::info!(
                "Removing {} stale downloaded_purchases entries (files deleted)",
                stale.len()
            );
            for (track_id, format_id) in &stale {
                let _ = self.conn.execute(
                    "DELETE FROM downloaded_purchases WHERE track_id = ?1 AND format_id = ?2",
                    rusqlite::params![track_id, format_id],
                );
            }
        }

        valid_ids.sort_unstable();
        valid_ids.dedup();
        Ok(valid_ids)
    }

    /// Get all downloaded (track_id, format_id) pairs for building per-format lookup.
    pub fn get_downloaded_purchase_formats(&self) -> Result<Vec<(i64, i64)>, LibraryError> {
        let mut stmt = self
            .conn
            .prepare("SELECT track_id, format_id FROM downloaded_purchases")
            .map_err(|e| LibraryError::Database(format!("Failed to prepare statement: {}", e)))?;

        let rows = stmt
            .query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)))
            .map_err(|e| {
                LibraryError::Database(format!("Failed to query downloaded purchases: {}", e))
            })?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| LibraryError::Database(format!("Failed to collect formats: {}", e)))
    }
}
