//! Album-scoped bulk operations: read all tracks for an album, reset a
//! track for re-download, and delete every row for an album atomically.

use super::row_to_cached_track_info;
use super::schema::OfflineCacheDb;
use crate::types::CachedTrackInfo;

impl OfflineCacheDb {
    /// Returns all cached track rows for a given album_id (any status).
    pub fn get_album_tracks(&self, album_id: &str) -> Result<Vec<CachedTrackInfo>, String> {
        let mut stmt = self
            .conn()
            .prepare(
                "SELECT track_id, title, artist, album, album_id, duration_secs,
                        file_size_bytes, quality, bit_depth, sample_rate, status,
                        progress_percent, error_message, created_at, last_accessed_at,
                        artwork_path, file_path
                 FROM cached_tracks WHERE album_id = ?1",
            )
            .map_err(|e| format!("Prepare failed: {}", e))?;
        let rows = stmt
            .query_map([album_id], row_to_cached_track_info)
            .map_err(|e| format!("Query failed: {}", e))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| format!("Row decode failed: {}", e))
    }

    /// Resets a track row to Pending state for re-download.
    /// Clears progress_percent and error_message.
    pub fn reset_track_for_redownload(&self, track_id: u64) -> Result<(), String> {
        self.conn()
            .execute(
                "UPDATE cached_tracks
                 SET status = 'queued', progress_percent = 0, error_message = NULL
                 WHERE track_id = ?1",
                [track_id as i64],
            )
            .map_err(|e| format!("Update failed: {}", e))?;
        Ok(())
    }

    /// Deletes all rows for the given album_id in a single transaction.
    /// Returns (deleted track_ids, total file_size_bytes freed).
    pub fn delete_album_tracks(&self, album_id: &str) -> Result<(Vec<u64>, u64), String> {
        let tx = self
            .conn()
            .unchecked_transaction()
            .map_err(|e| format!("Failed to begin tx: {}", e))?;

        let ids: Vec<u64> = {
            let mut stmt = tx
                .prepare("SELECT track_id FROM cached_tracks WHERE album_id = ?1")
                .map_err(|e| format!("Prepare failed: {}", e))?;
            let rows = stmt
                .query_map([album_id], |row| row.get::<_, i64>(0).map(|v| v as u64))
                .map_err(|e| format!("Query failed: {}", e))?;
            rows.collect::<rusqlite::Result<Vec<u64>>>()
                .map_err(|e| format!("Row decode failed: {}", e))?
        };

        let bytes: i64 = tx
            .query_row(
                "SELECT COALESCE(SUM(file_size_bytes), 0) FROM cached_tracks WHERE album_id = ?1",
                [album_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Sum failed: {}", e))?;

        tx.execute("DELETE FROM cached_tracks WHERE album_id = ?1", [album_id])
            .map_err(|e| format!("Delete failed: {}", e))?;

        tx.commit()
            .map_err(|e| format!("Commit failed: {}", e))?;

        Ok((ids, bytes as u64))
    }
}
