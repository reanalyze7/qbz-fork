//! Pure path/limit accessors that don't touch the DB.

use std::path::PathBuf;

use super::OfflineCacheState;

impl OfflineCacheState {
    /// Get the path for a track's audio file
    pub fn track_file_path(&self, track_id: u64, format: &str) -> PathBuf {
        let dir = self.cache_dir.read().unwrap();
        dir.join("tracks").join(format!("{}.{}", track_id, format))
    }

    /// Get the path for an album's artwork
    pub fn artwork_path(&self, album_id: &str) -> PathBuf {
        let dir = self.cache_dir.read().unwrap();
        dir.join("artwork").join(format!("{}.jpg", album_id))
    }

    /// Get the cache directory path
    pub fn get_cache_path(&self) -> String {
        let dir = self.cache_dir.read().unwrap();
        dir.to_string_lossy().to_string()
    }

    /// Seed the in-memory `limit_bytes` from a persisted value (read by the
    /// caller from the offline_settings DB) so the user's previously chosen
    /// limit survives across restarts. `None` keeps the in-memory default
    /// (5 GB) seeded by `new`/`init_at`.
    pub async fn apply_persisted_limit(&self, persisted: Option<u64>) {
        if let Some(bytes) = persisted {
            let mut limit = self.limit_bytes.lock().await;
            *limit = Some(bytes);
            log::info!(
                "Offline cache: applied persisted size limit ({} bytes)",
                bytes
            );
        } else {
            log::info!("Offline cache: no persisted size limit, keeping in-memory default");
        }
    }
}
