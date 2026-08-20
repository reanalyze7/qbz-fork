//! Deferred (background) opens for on-disk caches the rest of the app
//! already treats as optional.
//!
//! Startup audit 2026-08-20 (see STARTUP-AUDIT.md): the image-artwork cache
//! and the MusicBrainz cache were each opened synchronously in `main()`,
//! before the first window paint — a SQLite `Connection::open` + WAL pragma
//! + `CREATE TABLE IF NOT EXISTS` apiece. Both consumers already handle
//! "not yet open" as a normal degraded state:
//! - `artwork::cached_path_for` / friends walk `Option<ImageCacheService>`
//!   and just report a cache miss (falls back to network) when it's `None`.
//! - `AppRuntime::set_musicbrainz_cache` is a plain `Mutex<Option<_>>`
//!   setter; the MusicBrainz client "skips the cache when none is set"
//!   (see `qbz_integrations::musicbrainz`).
//!
//! So both opens move to a `tokio_rt.spawn` that runs concurrently with the
//! ~14k lines of closure wiring `main()` still has to do before
//! `window.show()`. The window can now go up before either SQLite file is
//! even opened; the caches fill in a few milliseconds later, same as any
//! other background job already running at that point (e.g. `offline_mode`
//! connectivity probing).

use crate::artwork::{self, ImageCache};
use qbz_app::shell::AppRuntime;
use qbz_models::FrontendAdapter;
use std::sync::{Arc, Mutex};

/// Publish an empty, shared image-cache handle immediately (every closure
/// captured later in `main()` clones this same `Arc`, so nothing downstream
/// changes), then open the SQLite store in the background and fill it in.
pub fn spawn_image_cache(tokio_rt: &tokio::runtime::Runtime) -> ImageCache {
    let image_cache: ImageCache = Arc::new(Mutex::new(None));
    artwork::set_shared_cache(image_cache.clone());
    let handle = image_cache.clone();
    tokio_rt.spawn(async move {
        match artwork::open_cache_blocking() {
            Some(service) => {
                if let Ok(mut guard) = handle.lock() {
                    *guard = Some(service);
                }
                log::info!("[qbz-slint] image cache opened (deferred)");
                artwork::spawn_evict(handle.clone());
            }
            None => log::warn!("[qbz-slint] image cache unavailable"),
        }
    });
    image_cache
}

/// Open the MusicBrainz metadata cache in the background and install it on
/// `runtime` once ready. Mirrors the pre-audit inline block in `main()`
/// verbatim, just moved off the synchronous startup path.
pub fn spawn_musicbrainz_cache<A: FrontendAdapter + Send + Sync + 'static>(
    tokio_rt: &tokio::runtime::Runtime,
    runtime: Arc<AppRuntime<A>>,
) {
    tokio_rt.spawn(async move {
        let Some(data_dir) = dirs::data_dir() else {
            return;
        };
        let cache_dir = data_dir.join("qbz").join("cache");
        if let Err(e) = std::fs::create_dir_all(&cache_dir) {
            log::warn!("[qbz-slint] MB cache dir create failed: {e}");
            return;
        }
        let db_path = cache_dir.join("musicbrainz_cache.db");
        match qbz_integrations::musicbrainz::cache::MusicBrainzCache::new(&db_path) {
            Ok(cache) => {
                runtime.core().set_musicbrainz_cache(cache);
                log::info!("[qbz-slint] MB cache opened at {db_path:?} (deferred)");
            }
            Err(e) => log::warn!("[qbz-slint] MB cache open failed: {e}"),
        }
    });
}
