//! Free-standing artwork pipeline glue for the queue/coverflow rows (not
//! `QueueController` methods).

mod apply;
mod reuse;

pub(super) use apply::apply_queue_art;
pub(super) use reuse::to_item_reuse;

use crate::AppWindow;

/// Where a resolved cover image should land.
#[derive(Clone, Copy)]
pub(super) enum ArtTarget {
    NowPlaying,
    Upcoming(usize),
    History(usize),
    /// A row in the single stable flat coverflow model, by flat index.
    CoverflowFlat(usize),
}

/// Resolve cover art for each job and apply it onto the matching row in
/// the `QueueState` global. One task per cover; misses are skipped.
pub(super) fn load_artwork(weak: slint::Weak<AppWindow>, jobs: Vec<(ArtTarget, String)>) {
    /// Decode size for all queue/coverflow covers (matches the artwork pipeline).
    const QUEUE_DECODE: u32 = 96;

    let Some(cache) = crate::artwork::shared_cache() else {
        return;
    };
    for (target, url) in jobs {
        let weak = weak.clone();
        let cache = cache.clone();
        // Source-aware: queue covers may be remote (Qobuz) OR local file
        // paths (Local Library / offline). Route file paths through
        // ArtworkRef::LocalFile (decode from disk).
        let art = if url.starts_with("http://") || url.starts_with("https://") {
            qbz_models::ArtworkRef::Remote(url)
        } else if let Some(p) = url.strip_prefix("file://") {
            qbz_models::ArtworkRef::LocalFile(p.to_string())
        } else {
            qbz_models::ArtworkRef::LocalFile(url)
        };

        // Decoded-pixel fast path: if this exact cover was already decoded at
        // this size (true for the covers still on screen after a one-position
        // shift), upload the cached pixels on the event loop and SKIP the tokio
        // decode entirely. This is the bulk of the per-click CPU-spike fix.
        let cache_key = match &art {
            qbz_models::ArtworkRef::Remote(u) => Some(u.clone()),
            qbz_models::ArtworkRef::LocalFile(p) => Some(p.clone()),
            _ => None,
        };
        if let Some(key) = cache_key.as_deref() {
            if let Some((pixels, w, h)) = crate::artwork::decoded_pixels(key, QUEUE_DECODE) {
                if let ArtTarget::CoverflowFlat(i) = target {
                    log::debug!("[coverflow-perf] cache-hit flat_idx={i}");
                }
                let weak = weak.clone();
                let _ = weak.upgrade_in_event_loop(move |win| {
                    let img = crate::artwork::pixels_to_image(&pixels, w, h);
                    apply_queue_art(&win, target, img);
                });
                continue;
            }
        }

        let perf_url = match target {
            ArtTarget::CoverflowFlat(i) => Some((i, cache_key.clone().unwrap_or_default())),
            _ => None,
        };
        tokio::spawn(async move {
            if let Some((i, u)) = perf_url.as_ref() {
                log::debug!("[coverflow-perf] decode flat_idx={i} url={u}");
            }
            let Some((pixels, w, h)) =
                crate::artwork::fetch_and_decode_ref(&art, &cache, QUEUE_DECODE).await
            else {
                return;
            };
            let _ = weak.upgrade_in_event_loop(move |win| {
                let img = crate::artwork::pixels_to_image(&pixels, w, h);
                apply_queue_art(&win, target, img);
            });
        });
    }
}
