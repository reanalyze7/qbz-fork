//! Cache-dir resolution, SQLite open + schema creation, and the private
//! `url_hash`/`cache_path` helpers shared by every other `impl` block.

use md5::{Digest, Md5};
use rusqlite::Connection;
use std::path::PathBuf;

use super::ImageCacheService;

impl ImageCacheService {
    pub fn new() -> Result<Self, String> {
        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| "Could not find cache directory".to_string())?
            .join("qbz")
            .join("images");

        std::fs::create_dir_all(&cache_dir)
            .map_err(|e| format!("Failed to create image cache dir: {}", e))?;

        let db_path = cache_dir.join("image_cache.db");
        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open image cache database: {}", e))?;

        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
            .map_err(|e| format!("Failed to enable WAL: {}", e))?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS cached_images (
                hash TEXT PRIMARY KEY,
                url TEXT NOT NULL,
                file_size INTEGER NOT NULL DEFAULT 0,
                last_accessed INTEGER NOT NULL DEFAULT 0
            );
            CREATE INDEX IF NOT EXISTS idx_last_accessed ON cached_images (last_accessed);",
        )
        .map_err(|e| format!("Failed to create image cache table: {}", e))?;

        Ok(Self { cache_dir, conn })
    }

    pub(super) fn url_hash(url: &str) -> String {
        let mut hasher = Md5::new();
        hasher.update(url.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub(super) fn cache_path(&self, hash: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.img", hash))
    }
}
