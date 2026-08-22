//! Per-track status/progress updates and deletion. Inserts live in
//! `tracks_insert.rs`, reads in `tracks_read.rs`.

use rusqlite::params;

use super::schema::OfflineCacheDb;
use crate::types::OfflineCacheStatus;

impl OfflineCacheDb {
    /// Update track status
    pub fn update_status(
        &self,
        track_id: u64,
        status: OfflineCacheStatus,
        error: Option<&str>,
    ) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE cached_tracks SET status = ?1, error_message = ?2 WHERE track_id = ?3",
                params![status.as_str(), error, track_id as i64],
            )
            .map_err(|e| format!("Failed to update status: {}", e))?;

        Ok(())
    }

    /// Update caching progress
    pub fn update_progress(
        &self,
        track_id: u64,
        progress: u8,
        size_bytes: u64,
    ) -> Result<(), String> {
        self.conn.execute(
            "UPDATE cached_tracks SET progress_percent = ?1, file_size_bytes = ?2 WHERE track_id = ?3",
            params![progress as i64, size_bytes as i64, track_id as i64],
        ).map_err(|e| format!("Failed to update progress: {}", e))?;

        Ok(())
    }

    /// Mark caching as complete
    pub fn mark_complete(&self, track_id: u64, file_size: u64) -> Result<(), String> {
        self.conn.execute(
            "UPDATE cached_tracks SET status = 'ready', progress_percent = 100, file_size_bytes = ?1, last_accessed_at = datetime('now') WHERE track_id = ?2",
            params![file_size as i64, track_id as i64],
        ).map_err(|e| format!("Failed to mark complete: {}", e))?;

        Ok(())
    }

    /// Update last accessed time (for LRU)
    pub fn touch(&self, track_id: u64) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE cached_tracks SET last_accessed_at = datetime('now') WHERE track_id = ?1",
                params![track_id as i64],
            )
            .map_err(|e| format!("Failed to update access time: {}", e))?;

        Ok(())
    }

    /// Delete a track from cache
    pub fn delete_track(&self, track_id: u64) -> Result<Option<String>, String> {
        // Get file path before deleting
        let file_path: Option<String> = self
            .conn
            .query_row(
                "SELECT file_path FROM cached_tracks WHERE track_id = ?1",
                params![track_id as i64],
                |row| row.get(0),
            )
            .ok();

        self.conn
            .execute(
                "DELETE FROM cached_tracks WHERE track_id = ?1",
                params![track_id as i64],
            )
            .map_err(|e| format!("Failed to delete track: {}", e))?;

        Ok(file_path)
    }

    /// Update file path for a track (after organizing)
    pub fn update_file_path(&self, track_id: u64, new_path: &str) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE cached_tracks SET file_path = ?1 WHERE track_id = ?2",
                params![new_path, track_id as i64],
            )
            .map_err(|e| format!("Failed to update file path: {}", e))?;
        Ok(())
    }

    /// Update artwork path for a track
    pub fn update_artwork_path(&self, track_id: u64, artwork_path: &str) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE cached_tracks SET artwork_path = ?1 WHERE track_id = ?2",
                params![artwork_path, track_id as i64],
            )
            .map_err(|e| format!("Failed to update artwork path: {}", e))?;
        Ok(())
    }
}
