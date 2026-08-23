use qbz_models::Quality;

/// The queue shape the decision needs, projected from `QbzCore::get_queue_state`
/// each tick: the current track id, the `(id, streamable)` upcoming list, the
/// repeat key and the stop-after marker.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct QueueSnapshot {
    pub current: u64,
    pub upcoming: Vec<(u64, bool)>,
    pub repeat: String,
    pub stop_after: Option<u64>,
    /// True when autoplay mode is "infinite" — logged-unsupported in P0 and
    /// treated as queue-finished (01 §3.1-5c).
    pub autoplay_infinite: bool,
}

/// Bounded skip-walk to the first streamable upcoming track. Returns
/// `(index, track_id)` of the first `playable == true` entry within `max_walk`
/// steps, or `None` when none is found inside the bound (never walks forever).
/// Mirrors `playback.rs::advance_to_playable`'s `MAX_OFFLINE_SKIPS` cap.
pub fn next_playable(upcoming: &[(u64, bool)], max_walk: usize) -> Option<(usize, u64)> {
    for (i, &(id, playable)) in upcoming.iter().enumerate() {
        if i >= max_walk {
            break;
        }
        if playable {
            return Some((i, id));
        }
    }
    None
}

/// Map the desktop `ui_prefs.streaming_quality` key to a request-layer
/// [`Quality`]. Byte-identical contract to `crates/qbz/src/ui_prefs.rs:823`
/// (`streaming_quality_for_key`), replicated here because the desktop crate is
/// out of the daemon's dependency graph. Unknown/unset keys fall back to the
/// top tier so hi-res never silently downgrades (01 §3.1).
pub fn quality_from_key(key: &str) -> Quality {
    match key {
        "mp3" => Quality::Mp3,
        "cd" => Quality::Lossless,
        "hires" => Quality::HiRes,
        _ => Quality::UltraHiRes, // "hires_plus" + unknown keys
    }
}
