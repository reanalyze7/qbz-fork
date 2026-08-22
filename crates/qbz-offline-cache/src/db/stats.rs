//! Aggregate/whole-table operations: stats for the cache-manager UI,
//! LRU eviction candidates, and full-cache clear.

use super::schema::OfflineCacheDb;
use crate::types::OfflineCacheStats;

impl OfflineCacheDb {
    /// Get statistics
    pub fn get_stats(
        &self,
        cache_path: &str,
        limit_bytes: Option<u64>,
    ) -> Result<OfflineCacheStats, String> {
        let total_tracks: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM cached_tracks", [], |row| row.get(0))
            .map_err(|e| format!("Failed to count tracks: {}", e))?;

        let ready_tracks: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM cached_tracks WHERE status = 'ready'",
                [],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to count ready tracks: {}", e))?;

        let downloading_tracks: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM cached_tracks WHERE status = 'downloading' OR status = 'queued'",
            [],
            |row| row.get(0),
        ).map_err(|e| format!("Failed to count downloading tracks: {}", e))?;

        let failed_tracks: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM cached_tracks WHERE status = 'failed'",
                [],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to count failed tracks: {}", e))?;

        let total_size: i64 = self.conn.query_row(
            "SELECT COALESCE(SUM(file_size_bytes), 0) FROM cached_tracks WHERE status = 'ready'",
            [],
            |row| row.get(0),
        ).map_err(|e| format!("Failed to sum sizes: {}", e))?;

        Ok(OfflineCacheStats {
            total_tracks: total_tracks as usize,
            ready_tracks: ready_tracks as usize,
            downloading_tracks: downloading_tracks as usize,
            failed_tracks: failed_tracks as usize,
            total_size_bytes: total_size as u64,
            limit_bytes,
            cache_path: cache_path.to_string(),
        })
    }

    /// Get tracks to evict (LRU order) to free up space
    pub fn get_tracks_for_eviction(
        &self,
        bytes_to_free: u64,
    ) -> Result<Vec<(u64, String)>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT track_id, file_path, file_size_bytes FROM cached_tracks
             WHERE status = 'ready'
             ORDER BY last_accessed_at ASC",
            )
            .map_err(|e| format!("Failed to prepare eviction query: {}", e))?;

        let mut result = Vec::new();
        let mut freed = 0u64;

        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)? as u64,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)? as u64,
                ))
            })
            .map_err(|e| format!("Failed to query for eviction: {}", e))?;

        for row in rows {
            if freed >= bytes_to_free {
                break;
            }
            let (track_id, file_path, size) =
                row.map_err(|e| format!("Failed to read row: {}", e))?;
            result.push((track_id, file_path));
            freed += size;
        }

        Ok(result)
    }

    /// Clear all entries
    pub fn clear_all(&self) -> Result<Vec<String>, String> {
        // Get all file paths first
        let mut stmt = self
            .conn
            .prepare("SELECT file_path FROM cached_tracks")
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let paths: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| format!("Failed to query paths: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        self.conn
            .execute("DELETE FROM cached_tracks", [])
            .map_err(|e| format!("Failed to clear database: {}", e))?;

        Ok(paths)
    }
}
