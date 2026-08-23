//! Shared mutable state: the currently-loaded mix track list + its
//! metadata/shuffle/index helpers.

use std::sync::{LazyLock, Mutex};

use qbz_models::Track;

/// The currently-loaded mix track list, kept so play-all / per-track
/// play can build the queue without re-fetching.
pub(super) static CURRENT_MIX: LazyLock<Mutex<Vec<Track>>> = LazyLock::new(|| Mutex::new(Vec::new()));

pub fn mix_meta(kind: &str) -> (&'static str, String) {
    match kind {
        "daily" => (
            "DailyQ",
            qbz_i18n::t("Elevate your day with a customized selection of music."),
        ),
        "weekly" => ("WeeklyQ", qbz_i18n::t("A fresh mix every week.")),
        "fav" => ("FavQ", qbz_i18n::t("A fresh shuffle from your personal library.")),
        "top" => ("TopQ", qbz_i18n::t("From your most-played playlists.")),
        _ => ("Mix", String::new()),
    }
}

/// Lightweight, deterministic-per-call shuffle (no rng dep).
pub(super) fn shuffle(tracks: &mut [Track]) {
    let mut seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(1)
        | 1;
    let n = tracks.len();
    for i in (1..n).rev() {
        // xorshift
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        let j = (seed % (i as u64 + 1)) as usize;
        tracks.swap(i, j);
    }
}

/// The cached mix track list (for play-all / per-track play).
pub fn current_tracks() -> Vec<Track> {
    CURRENT_MIX.lock().map(|c| c.clone()).unwrap_or_default()
}

/// The current mix tracks in a fresh random order (for the Shuffle
/// action) — does not mutate the displayed list.
pub fn shuffled_tracks() -> Vec<Track> {
    let mut tracks = current_tracks();
    shuffle(&mut tracks);
    tracks
}

/// Index of `track_id` within the current mix (for play-from-here).
pub fn index_of(track_id: &str) -> usize {
    CURRENT_MIX
        .lock()
        .ok()
        .and_then(|c| c.iter().position(|t| t.id.to_string() == track_id))
        .unwrap_or(0)
}
