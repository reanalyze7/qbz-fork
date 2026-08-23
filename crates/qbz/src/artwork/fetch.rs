//! The network/disk I/O layer: resolving a URL or local path to raw RGBA8
//! pixels, going through the disk cache and the decoded-pixel LRU.

use std::sync::Arc;

use qbz_models::ArtworkRef;

use super::cache::ImageCache;
use super::decode::{decode_rgba, decoded_pixels, store_decoded};

/// Total-request timeout: without one, a half-up network (DNS blackhole,
/// captive portal) pins a `MAX_CONCURRENT` semaphore permit indefinitely.
const HTTP_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

/// Process-wide async HTTP client for artwork downloads. A single client pools
/// connections / reuses keep-alive across all concurrent artwork jobs
/// (`MAX_CONCURRENT` = 16), instead of `reqwest::get` building a fresh client +
/// connection pool on every cache miss (the fd-churn / deferred EMFILE risk).
/// rustls per the workspace default; if the builder fails, fall back to the
/// default client (equivalent to `Client::new()`), keeping this infallible —
/// matching the silent-fallback style of `open_cache`.
static HTTP: std::sync::LazyLock<reqwest::Client> = std::sync::LazyLock::new(|| {
    reqwest::Client::builder()
        .timeout(HTTP_TIMEOUT)
        .build()
        .unwrap_or_default()
});

/// Resolve a remote URL to raw bytes via the shared disk cache: a hit reads
/// from disk, a miss downloads and stores. HTTP(S) only.
///
/// Genuinely-INTERNET fetches (Qobuz CDN covers) are gated while offline
/// mode is active — offline means zero internet traffic, so a miss fails
/// soft to the placeholder. Disk hits always serve regardless.
async fn fetch_cached_http(url: &str, cache: &ImageCache) -> Option<Vec<u8>> {
    let cached_path = {
        let guard = cache.lock().ok()?;
        guard.as_ref().and_then(|service| service.get(url))
    };

    match cached_path {
        Some(path) => tokio::fs::read(&path).await.ok(),
        // Offline: an internet miss must not burn a network attempt (or pin a
        // semaphore permit) — fail soft to the placeholder; nothing negative
        // is cached, so the cover retries naturally once back online.
        None if crate::offline_mode::engine().is_offline() => None,
        None => {
            let downloaded = HTTP.get(url).send().await.ok()?.bytes().await.ok()?.to_vec();
            if let Ok(guard) = cache.lock() {
                if let Some(service) = guard.as_ref() {
                    let _ = service.store(url, &downloaded);
                }
            }
            Some(downloaded)
        }
    }
}

/// Resolve an [`ArtworkRef`] to raw RGBA8 pixels, downscaled to
/// `decode_size`, regardless of origin. This is the source-aware entry
/// point that fixes local artwork never reaching the UI: HTTP thumbnails go
/// through the disk cache, local files are read directly, and embedded bytes
/// decode in place. Runs on a worker thread; the result tuple is `Send`.
pub async fn fetch_and_decode_ref(
    art: &ArtworkRef,
    cache: &ImageCache,
    decode_size: u32,
) -> Option<(Vec<u8>, u32, u32)> {
    if art.is_empty() {
        return None;
    }

    // Decoded-pixel cache key: the stable resolved location for this art at this
    // decode size. A hit returns the already-decoded RGBA tuple and skips both
    // the disk read AND the `image::load_from_memory().thumbnail()` decode — this
    // is what makes a one-position queue/coverflow shift near-free (the 6 covers
    // still on screen reuse their decoded pixels instead of being re-decoded).
    // `Embedded` has no stable URL, so it is never decode-cached.
    let cache_key: Option<String> = match art {
        ArtworkRef::None | ArtworkRef::Embedded(_) => None,
        ArtworkRef::LocalFile(path) => Some(path.clone()),
        ArtworkRef::Remote(url) => Some(url.clone()),
    };
    if let Some(key) = cache_key.as_deref() {
        if let Some((pixels, w, h)) = decoded_pixels(key, decode_size) {
            return Some(((*pixels).clone(), w, h));
        }
    }

    let bytes: Vec<u8> = match art {
        ArtworkRef::None => return None,
        ArtworkRef::Embedded(b) => b.clone(),
        ArtworkRef::LocalFile(path) => tokio::fs::read(path).await.ok()?,
        ArtworkRef::Remote(url) => fetch_cached_http(url, cache).await?,
    };
    let (pixels, w, h) = decode_rgba(&bytes, decode_size)?;
    if let Some(key) = cache_key {
        store_decoded(&key, decode_size, &(Arc::new(pixels.clone()), w, h));
    }
    Some((pixels, w, h))
}

/// Resolve one cover image (by remote URL) to raw RGBA8 pixels. Kept for the
/// many card/row jobs that already hold a URL; source-aware call sites
/// (local library) use [`fetch_and_decode_ref`].
pub async fn fetch_and_decode(
    url: &str,
    cache: &ImageCache,
    decode_size: u32,
) -> Option<(Vec<u8>, u32, u32)> {
    fetch_and_decode_ref(&ArtworkRef::Remote(url.to_string()), cache, decode_size).await
}
