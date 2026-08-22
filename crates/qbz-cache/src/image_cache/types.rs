//! The [`ImageCacheService`] struct and [`ImageCacheStats`] type. Methods
//! live in sibling `impl` blocks across `open.rs`/`access.rs`/`maintenance.rs`.

use rusqlite::Connection;
use std::path::PathBuf;

/// Cache statistics.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ImageCacheStats {
    pub total_bytes: u64,
    pub file_count: u64,
}

pub struct ImageCacheService {
    pub(super) cache_dir: PathBuf,
    pub(super) conn: Connection,
}
