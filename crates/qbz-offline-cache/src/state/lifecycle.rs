//! Opening/closing the DB(s) and creating cache directories.

use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tokio::sync::{Mutex, Semaphore};

use crate::db::OfflineCacheDb;
use crate::downloader::StreamFetcher;

use super::{OfflineCacheState, DEFAULT_LIMIT_BYTES};

impl OfflineCacheState {
    /// Initialize the offline cache at the platform cache dir.
    pub fn new() -> Result<Self, String> {
        let cache_dir = dirs::cache_dir()
            .ok_or("Could not determine cache directory")?
            .join("qbz")
            .join("audio");

        // Create directories
        std::fs::create_dir_all(&cache_dir)
            .map_err(|e| format!("Failed to create cache directory: {}", e))?;
        std::fs::create_dir_all(cache_dir.join("tracks"))
            .map_err(|e| format!("Failed to create tracks directory: {}", e))?;
        std::fs::create_dir_all(cache_dir.join("artwork"))
            .map_err(|e| format!("Failed to create artwork directory: {}", e))?;

        let db_path = cache_dir.join("index.db");
        let db = OfflineCacheDb::new(&db_path)?;

        let state = Self {
            db: Arc::new(Mutex::new(Some(db))),
            fetcher: Arc::new(StreamFetcher::new()),
            cache_dir: Arc::new(RwLock::new(cache_dir.clone())),
            limit_bytes: Arc::new(Mutex::new(Some(DEFAULT_LIMIT_BYTES))),
            cache_semaphore: Arc::new(Semaphore::new(3)),
            library_db: Arc::new(Mutex::new(None)),
        };

        log::info!("Offline cache initialized at: {:?}", cache_dir);

        Ok(state)
    }

    pub fn new_empty() -> Self {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("qbz")
            .join("audio");
        Self {
            db: Arc::new(Mutex::new(None)),
            fetcher: Arc::new(StreamFetcher::new()),
            cache_dir: Arc::new(RwLock::new(cache_dir)),
            limit_bytes: Arc::new(Mutex::new(Some(DEFAULT_LIMIT_BYTES))),
            cache_semaphore: Arc::new(Semaphore::new(3)),
            library_db: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn init_at(&self, cache_base_dir: &std::path::Path) -> Result<(), String> {
        let cache_dir = cache_base_dir.join("audio");
        std::fs::create_dir_all(&cache_dir)
            .map_err(|e| format!("Failed to create cache directory: {}", e))?;
        std::fs::create_dir_all(cache_dir.join("tracks"))
            .map_err(|e| format!("Failed to create tracks directory: {}", e))?;
        std::fs::create_dir_all(cache_dir.join("artwork"))
            .map_err(|e| format!("Failed to create artwork directory: {}", e))?;
        let db_path = cache_dir.join("index.db");
        let new_db = OfflineCacheDb::new(&db_path)?;
        let mut guard = self.db.lock().await;
        *guard = Some(new_db);
        // Update cache_dir to user-scoped path
        if let Ok(mut dir_guard) = self.cache_dir.write() {
            *dir_guard = cache_dir.clone();
        }
        log::info!("Offline cache initialized at: {:?}", cache_dir);
        Ok(())
    }

    /// Open a separate library DB connection for download post-processing.
    /// Must be called after library.init_at() so the schema exists.
    pub async fn init_library_connection(&self, data_dir: &std::path::Path) -> Result<(), String> {
        let db_path = data_dir.join("library.db");
        let lib_db = qbz_library::LibraryDatabase::open(&db_path)
            .map_err(|e| format!("Failed to open download library connection: {}", e))?;
        let mut guard = self.library_db.lock().await;
        *guard = Some(lib_db);
        log::info!(
            "Offline cache: separate library DB connection opened at {:?}",
            db_path
        );
        Ok(())
    }

    pub async fn teardown(&self) {
        // Close library connection first (before main teardown)
        {
            let mut lib_guard = self.library_db.lock().await;
            *lib_guard = None;
        }
        let mut guard = self.db.lock().await;
        *guard = None;
    }
}
