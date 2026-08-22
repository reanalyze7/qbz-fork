//! Size-management operations: `evict`, `stats`, `clear`.

use rusqlite::params;

use super::{ImageCacheService, ImageCacheStats};

impl ImageCacheService {
    /// Evict least-recently-accessed entries until total size is under max_bytes.
    pub fn evict(&self, max_bytes: u64) -> Result<u64, String> {
        let total: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(file_size), 0) FROM cached_images",
                [],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to query cache size: {}", e))?;

        if (total as u64) <= max_bytes {
            return Ok(0);
        }

        let mut to_free = (total as u64) - max_bytes;
        let mut freed: u64 = 0;

        // Get LRU entries (oldest access first)
        let mut stmt = self
            .conn
            .prepare("SELECT hash, file_size FROM cached_images ORDER BY last_accessed ASC")
            .map_err(|e| format!("Failed to prepare eviction query: {}", e))?;

        let entries: Vec<(String, i64)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| format!("Failed to query LRU entries: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        for (hash, file_size) in entries {
            if to_free == 0 {
                break;
            }
            let path = self.cache_path(&hash);
            if path.exists() {
                let _ = std::fs::remove_file(&path);
            }
            let _ = self
                .conn
                .execute("DELETE FROM cached_images WHERE hash = ?1", params![hash]);
            let size = file_size as u64;
            freed += size;
            to_free = to_free.saturating_sub(size);
        }

        Ok(freed)
    }

    /// Get cache statistics.
    pub fn stats(&self) -> Result<ImageCacheStats, String> {
        let (total_bytes, file_count): (i64, i64) = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(file_size), 0), COUNT(*) FROM cached_images",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|e| format!("Failed to query image cache stats: {}", e))?;

        Ok(ImageCacheStats {
            total_bytes: total_bytes as u64,
            file_count: file_count as u64,
        })
    }

    /// Clear the entire cache.
    pub fn clear(&self) -> Result<u64, String> {
        let stats = self.stats()?;

        // Delete all files
        if let Ok(entries) = std::fs::read_dir(&self.cache_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "img").unwrap_or(false) {
                    let _ = std::fs::remove_file(path);
                }
            }
        }

        // Clear database
        self.conn
            .execute("DELETE FROM cached_images", [])
            .map_err(|e| format!("Failed to clear image cache table: {}", e))?;

        Ok(stats.total_bytes)
    }
}
