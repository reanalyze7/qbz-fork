//! The per-image read/write paths: `get` and `store`.

use rusqlite::params;
use std::path::PathBuf;

use super::ImageCacheService;

impl ImageCacheService {
    /// Get a cached image path, updating last-access time.
    /// Returns None if not cached.
    pub fn get(&self, url: &str) -> Option<PathBuf> {
        let hash = Self::url_hash(url);
        let path = self.cache_path(&hash);

        if !path.exists() {
            // File missing — clean up stale DB entry
            let _ = self
                .conn
                .execute("DELETE FROM cached_images WHERE hash = ?1", params![hash]);
            return None;
        }

        // Update last-accessed time
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let _ = self.conn.execute(
            "UPDATE cached_images SET last_accessed = ?1 WHERE hash = ?2",
            params![now, hash],
        );

        Some(path)
    }

    /// Store image bytes in the cache.
    /// Returns the local file path on success.
    pub fn store(&self, url: &str, bytes: &[u8]) -> Result<PathBuf, String> {
        let hash = Self::url_hash(url);
        let path = self.cache_path(&hash);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        std::fs::write(&path, bytes).map_err(|e| format!("Failed to write cached image: {}", e))?;

        let file_size = bytes.len() as i64;
        self.conn
            .execute(
                "INSERT OR REPLACE INTO cached_images (hash, url, file_size, last_accessed)
                 VALUES (?1, ?2, ?3, ?4)",
                params![hash, url, file_size, now],
            )
            .map_err(|e| format!("Failed to insert image cache entry: {}", e))?;

        Ok(path)
    }
}
