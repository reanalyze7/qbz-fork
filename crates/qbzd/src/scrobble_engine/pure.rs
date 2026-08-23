use std::time::{SystemTime, UNIX_EPOCH};

use qbz_models::QueueTrack;

/// Whether the current track is due to scrobble now: it has a threshold, has
/// been played to it, and hasn't been scrobbled yet. Pure — unit-tested.
pub(super) fn due(position_secs: u64, threshold: Option<u64>, scrobbled: bool) -> bool {
    !scrobbled && threshold.is_some_and(|t| position_secs >= t)
}

/// The album name, unless it's empty or the queue-track "Unknown Album"
/// placeholder (both scrobble better as "no album" than a fake one).
pub(super) fn album_opt(t: &QueueTrack) -> Option<&str> {
    if t.album.is_empty() || t.album == "Unknown Album" {
        None
    } else {
        Some(&t.album)
    }
}

pub(super) fn now_unix() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}
