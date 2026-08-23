//! The semaphore-bounded spawn functions: one per artwork-source shape
//! (remote-only, local-only, mixed).

use std::sync::Arc;

use qbz_models::ArtworkRef;
use tokio::sync::Semaphore;

use super::ArtworkJob;
use crate::artwork::apply::apply_artwork;
use crate::artwork::cache::ImageCache;
use crate::artwork::fetch::{fetch_and_decode, fetch_and_decode_ref};
use crate::artwork::target::ArtworkTarget;
use crate::AppWindow;

/// Cap on simultaneous artwork downloads.
const MAX_CONCURRENT: usize = 16;

/// Spawn artwork downloads for every job. Each completion updates only its
/// own card row. Must be called from within the tokio runtime.
pub fn spawn_loads(jobs: Vec<ArtworkJob>, window: slint::Weak<AppWindow>, cache: ImageCache) {
    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT));
    for job in jobs {
        let semaphore = semaphore.clone();
        let window = window.clone();
        let cache = cache.clone();
        tokio::spawn(async move {
            let _permit = semaphore.acquire().await.ok()?;
            let decode_size = job.target.decode_size();
            let Some((pixels, width, height)) =
                fetch_and_decode(&job.url, &cache, decode_size).await
            else {
                // Failed fetch/decode never reaches apply_artwork — free the
                // windowed-dispatch dedupe slot here so a later band pass can
                // retry this cover instead of skipping it for the session.
                if let ArtworkTarget::FavoriteAlbumById { id, .. } = &job.target {
                    crate::favorites::album_artwork_job_done(id);
                }
                return None;
            };
            let target = job.target;
            let url = job.url;
            let _ = window.upgrade_in_event_loop(move |w| {
                apply_artwork(&w, target, &url, &pixels, width, height);
            });
            Some(())
        });
    }
}

/// Like `spawn_loads`, but each job's `url` is a LOCAL filesystem path
/// (Local Library covers) rather than an HTTP URL. Decodes via the
/// source-aware `ArtworkRef::LocalFile` instead of the HTTP cache path.
pub fn spawn_local_loads(jobs: Vec<ArtworkJob>, window: slint::Weak<AppWindow>, cache: ImageCache) {
    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT));
    for job in jobs {
        let semaphore = semaphore.clone();
        let window = window.clone();
        let cache = cache.clone();
        tokio::spawn(async move {
            let _permit = semaphore.acquire().await.ok()?;
            let decode_size = job.target.decode_size();
            let art = ArtworkRef::LocalFile(job.url.clone());
            let Some((pixels, width, height)) =
                fetch_and_decode_ref(&art, &cache, decode_size).await
            else {
                // Failed fetch/decode never reaches apply_artwork — free the
                // windowed-dispatch dedupe slot here so a later band pass can
                // retry this cover instead of skipping it for the session.
                if let ArtworkTarget::LocalAlbumById { id, .. } = &job.target {
                    crate::local_library::album_artwork_job_done(id);
                }
                return None;
            };
            let target = job.target;
            let url = job.url;
            let _ = window.upgrade_in_event_loop(move |w| {
                apply_artwork(&w, target, &url, &pixels, width, height);
            });
            Some(())
        });
    }
}

/// Artwork dispatch for the SEARCH cortinilla, whose rows mix two sources in a
/// single payload: Qobuz catalog covers (http(s) URLs) and Local Library
/// covers (absolute filesystem paths). Each job is routed by its url's shape —
/// http → the HTTP cache path (gated offline, like Qobuz CDN covers);
/// anything else → `LocalFile` (`fs::read`).
pub fn spawn_search_loads(jobs: Vec<ArtworkJob>, window: slint::Weak<AppWindow>, cache: ImageCache) {
    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT));
    for job in jobs {
        let semaphore = semaphore.clone();
        let window = window.clone();
        let cache = cache.clone();
        tokio::spawn(async move {
            let _permit = semaphore.acquire().await.ok()?;
            let decode_size = job.target.decode_size();
            let is_http = job.url.starts_with("http://") || job.url.starts_with("https://");
            let (pixels, width, height) = if is_http {
                // Qobuz CDN cover (internet) — offline-gated inside fetch_and_decode.
                fetch_and_decode(&job.url, &cache, decode_size).await?
            } else {
                let art = ArtworkRef::LocalFile(job.url.clone());
                fetch_and_decode_ref(&art, &cache, decode_size).await?
            };
            let target = job.target;
            let url = job.url;
            let _ = window.upgrade_in_event_loop(move |w| {
                apply_artwork(&w, target, &url, &pixels, width, height);
            });
            Some(())
        });
    }
}
