//! Album-level metadata cache fed by the playback album-fetch paths.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

/// Album-level metadata captured when an album is fetched for playback,
/// keyed by album id. The queue track itself (a `qbz_models::QueueTrack`)
/// carries no genre / release-date and — for the `album/get` path — no
/// per-track quality, so `record_recent` looks the album up here to stamp
/// the Recently Played card with genre, release date, and quality badge.
/// Matches Tauri's `album_to_card_meta`, which reads these off the `Album`.
#[derive(Clone, Debug, Default)]
pub struct AlbumMeta {
    pub genre: String,
    /// Raw ISO release date (e.g. "2021-05-14"); localized at render time.
    pub release_date: String,
    /// "hires" | "cd" | "" — drives the album card quality badge.
    pub quality_tier: String,
    /// "Hi-Res: 24-bit / 96 kHz" — quality badge hover tooltip.
    pub quality_label: String,
}

static ALBUM_META: LazyLock<Mutex<HashMap<String, AlbumMeta>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Cache album-level metadata for `album_id`, so a subsequent play of any
/// of its tracks records genre / release date / quality on the recent card.
/// Called from the playback album-fetch paths.
pub fn remember_album_meta(album_id: &str, meta: AlbumMeta) {
    if album_id.is_empty() {
        return;
    }
    if let Ok(mut map) = ALBUM_META.lock() {
        map.insert(album_id.to_string(), meta);
    }
}

/// Look up cached album-level metadata for `album_id` (if any).
pub fn album_meta(album_id: &str) -> Option<AlbumMeta> {
    ALBUM_META.lock().ok().and_then(|map| map.get(album_id).cloned())
}
