//! Shared mutable state: the resolved image cache and the live search query.

use std::sync::Mutex as StdMutex;
use std::sync::OnceLock;

use chrono::{DateTime, Utc};

use crate::artwork::ImageCache;

/// Shared image cache for resolving blocked-album cover thumbnails (the artist
/// tab has no covers; the album tab does). Set once during startup wiring.
pub(super) static IMAGE_CACHE: OnceLock<ImageCache> = OnceLock::new();

/// Store the shared image cache for album-cover resolution (idempotent).
pub fn set_image_cache(cache: ImageCache) {
    let _ = IMAGE_CACHE.set(cache);
}

/// The live search query (Rust-side source of truth). The view echoes it back
/// from `BlacklistState.search-query`; this is what `refilter` reads so a
/// toggle/remove/clear re-push keeps the current filter applied.
static QUERY: StdMutex<String> = StdMutex::new(String::new());

pub(super) fn current_query() -> String {
    QUERY.lock().map(|q| q.clone()).unwrap_or_default()
}

pub(super) fn set_query(q: String) {
    if let Ok(mut guard) = QUERY.lock() {
        *guard = q;
    }
}

/// Format a unix-seconds timestamp as "MMM D, YYYY" (English). Falls back to an
/// empty string for a non-representable value.
pub(super) fn format_added(secs: i64) -> String {
    DateTime::<Utc>::from_timestamp(secs, 0)
        .map(|dt| dt.format("%b %-d, %Y").to_string())
        .unwrap_or_default()
}
