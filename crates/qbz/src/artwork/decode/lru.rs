//! The decoded-pixel LRU: repeat decodes of the same `(url, size)` become a
//! HashMap hit instead of a full `image::load_from_memory().thumbnail()`.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use super::DecodedPixels;

/// Decoded-pixel LRU. Repeat decodes of the same `(url, size)` — exactly what
/// the coverflow / queue refresh hammered every click — become a HashMap hit +
/// a cheap pixel upload instead of a full `image::load_from_memory().thumbnail()`.
/// Byte-budgeted (large now-playing decodes run ~1.44MB each as RGBA, so an
/// entry cap alone let a long shuffle session grow unbounded); the entry cap
/// stays as a backstop for many tiny entries. Insertion order approximates LRU
/// (re-insert on hit moves the entry to the back).
const DECODED_CACHE_CAP: usize = 256;

/// Byte budget for the decoded-pixel cache: 48MB, lowered to 24MB on
/// small-RAM machines (< 8GB `MemTotal` per `/proc/meminfo`, read once;
/// non-Linux / unreadable falls back to the default).
static DECODED_CACHE_BUDGET: LazyLock<usize> = LazyLock::new(|| {
    const DEFAULT: usize = 48 * 1024 * 1024;
    const SMALL: usize = 24 * 1024 * 1024;
    let small_ram = std::fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("MemTotal:"))
                .and_then(|l| l.split_whitespace().nth(1)?.parse::<u64>().ok())
        })
        .map(|kb| kb < 8 * 1024 * 1024)
        .unwrap_or(false);
    if small_ram {
        SMALL
    } else {
        DEFAULT
    }
});

struct DecodedCache {
    /// `(url, size)` -> decoded pixels. Insertion order = eviction order.
    map: HashMap<(String, u32), DecodedPixels>,
    /// Keys in insertion order; the front is the eviction candidate.
    order: Vec<(String, u32)>,
    /// Total pixel bytes held (`w*h*4` per entry), checked against
    /// `DECODED_CACHE_BUDGET` on insert.
    bytes: usize,
}

static DECODED_PIXEL_CACHE: LazyLock<Mutex<DecodedCache>> = LazyLock::new(|| {
    Mutex::new(DecodedCache {
        map: HashMap::new(),
        order: Vec::new(),
        bytes: 0,
    })
});

/// Decoded-pixel cache lookup for `(url, size)`. A hit returns the shared RGBA
/// tuple — callers build the `slint::Image` via [`super::pixels_to_image`] on
/// the event loop and SKIP the expensive decode entirely.
pub fn decoded_pixels(url: &str, size: u32) -> Option<DecodedPixels> {
    let mut cache = DECODED_PIXEL_CACHE.lock().ok()?;
    let key = (url.to_string(), size);
    let hit = cache.map.get(&key).cloned();
    if hit.is_some() {
        // Move to the back (most-recently-used).
        if let Some(pos) = cache.order.iter().position(|k| k == &key) {
            cache.order.remove(pos);
        }
        cache.order.push(key);
    }
    hit
}

/// Store decoded pixels for `(url, size)`, evicting LRU entries until both
/// the byte budget and the entry-count backstop hold.
pub(in crate::artwork) fn store_decoded(url: &str, size: u32, pixels: &DecodedPixels) {
    let Ok(mut cache) = DECODED_PIXEL_CACHE.lock() else {
        return;
    };
    let key = (url.to_string(), size);
    let entry_bytes = pixels.0.len();
    match cache.map.insert(key.clone(), pixels.clone()) {
        None => {
            cache.order.push(key);
            cache.bytes += entry_bytes;
        }
        Some(old) => {
            // Refresh recency + swap the byte accounting on an overwrite.
            cache.bytes = cache.bytes.saturating_sub(old.0.len()) + entry_bytes;
            if let Some(pos) = cache.order.iter().position(|k| k == &key) {
                cache.order.remove(pos);
                cache.order.push(key);
            }
        }
    }
    // Never evict the just-inserted entry (it sits at the back; the len > 1
    // guard covers the degenerate single-oversized-entry case).
    while (cache.bytes > *DECODED_CACHE_BUDGET || cache.order.len() > DECODED_CACHE_CAP)
        && cache.order.len() > 1
    {
        let oldest = cache.order.remove(0);
        if let Some(evicted) = cache.map.remove(&oldest) {
            cache.bytes = cache.bytes.saturating_sub(evicted.0.len());
        }
    }
}

