//! The offline-cache state holder (moved from `src-tauri/src/offline_cache/mod.rs`).
//!
//! A plain struct (no Tauri): the open SQLite index, the stream fetcher,
//! the cache-dir path, the size limit, the download concurrency semaphore,
//! and a separate library-DB connection for download post-processing.
//! Both the Tauri frontend (`tauri::State`) and the Slint frontend own one.
//!
//! Split into `lifecycle` (open/close the DB(s), create directories) and
//! `paths` (pure path/limit accessors) as `impl OfflineCacheState` blocks.

mod lifecycle;
mod paths;

use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tokio::sync::{Mutex, Semaphore};

use crate::db::OfflineCacheDb;
use crate::downloader::StreamFetcher;

/// Default cache size limit: 5 GB, used by both `new` and `new_empty`.
pub(super) const DEFAULT_LIMIT_BYTES: u64 = 5 * 1024 * 1024 * 1024;

/// Offline cache state manager
pub struct OfflineCacheState {
    pub db: Arc<Mutex<Option<OfflineCacheDb>>>,
    pub fetcher: Arc<StreamFetcher>,
    pub cache_dir: Arc<RwLock<PathBuf>>,
    /// Cache limit in bytes (None = unlimited)
    pub limit_bytes: Arc<Mutex<Option<u64>>>,
    pub cache_semaphore: Arc<Semaphore>,
    /// Separate library DB connection for download post-processing writes.
    /// This avoids contending with the main library DB mutex used by UI queries.
    pub library_db: Arc<Mutex<Option<qbz_library::LibraryDatabase>>>,
}
