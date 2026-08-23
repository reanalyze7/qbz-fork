//! The disk-cache surface: opening the shared QBZ image cache, the
//! process-wide handle, and cache-only lookups (no network).

use std::sync::{Arc, Mutex};

use qbz_cache::ImageCacheService;
use qbz_models::ArtworkRef;

/// Default image-cache size budget (matches the Tauri default).
pub const MAX_CACHE_BYTES: u64 = 200 * 1024 * 1024;

/// Shared, optional image cache. `None` when the cache could not be opened
/// — artwork then falls back to direct downloads.
pub type ImageCache = Arc<Mutex<Option<ImageCacheService>>>;

/// Open the shared QBZ image cache.
pub fn open_cache() -> ImageCache {
    Arc::new(Mutex::new(open_cache_blocking()))
}

/// Same SQLite open as [`open_cache`] (WAL pragma + `CREATE TABLE IF NOT
/// EXISTS`), without the `Arc<Mutex<..>>` wrapper — for callers that already
/// hold the shared handle and only need to fill it in (startup audit
/// 2026-08-20: lets the open happen on a background task instead of
/// blocking the first paint). `None` on failure, same degraded-mode
/// contract as an unopened cache: lookups miss, downloads still work.
pub fn open_cache_blocking() -> Option<ImageCacheService> {
    match ImageCacheService::new() {
        Ok(service) => Some(service),
        Err(e) => {
            log::warn!("[qbz-slint] image cache unavailable: {e}");
            None
        }
    }
}

/// Process-wide handle to the image cache, so controllers that are not
/// threaded the cache explicitly (the playback controller) can still
/// resolve cover art. Set once at startup.
static SHARED_CACHE: std::sync::OnceLock<ImageCache> = std::sync::OnceLock::new();

/// Publish the image cache for `shared_cache()` consumers. Call once.
pub fn set_shared_cache(cache: ImageCache) {
    let _ = SHARED_CACHE.set(cache);
}

/// The shared image cache, if `set_shared_cache` has run.
pub fn shared_cache() -> Option<ImageCache> {
    SHARED_CACHE.get().cloned()
}

/// Disk-cache lookup for a remote artwork URL: the cached file's path, or
/// `None` on miss / unopened cache. Never touches the network — offline
/// consumers (MPRIS art, artwork save-as) use this instead of downloading.
pub fn cached_path_for(url: &str) -> Option<std::path::PathBuf> {
    let cache = shared_cache()?;
    let guard = cache.lock().ok()?;
    guard.as_ref()?.get(url)
}

/// `file://` form of [`cached_path_for`], for the MPRIS `artUrl` property.
pub fn cached_file_url_for(url: &str) -> Option<String> {
    let path = cached_path_for(url)?;
    ArtworkRef::LocalFile(path.to_string_lossy().into_owned()).to_mpris_url()
}

/// Trim the image cache to the size budget. Runs once at startup.
pub fn spawn_evict(cache: ImageCache) {
    tokio::spawn(async move {
        if let Ok(guard) = cache.lock() {
            if let Some(service) = guard.as_ref() {
                match service.evict(MAX_CACHE_BYTES) {
                    Ok(freed) if freed > 0 => {
                        log::info!("[qbz-slint] image cache evicted {freed} bytes")
                    }
                    Ok(_) => {}
                    Err(e) => log::warn!("[qbz-slint] image cache eviction failed: {e}"),
                }
            }
        }
    });
}
