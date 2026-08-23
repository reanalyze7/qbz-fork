//! "Pick from currently visible/loaded set" helpers: the play-ready queue
//! source (`FAV_CURRENT`, written by `apply::tracks`) and the per-tab random
//! pick used by each tab's Shuffle / random button.

mod visible;

pub use visible::{
    random_visible_album, random_visible_artist, random_visible_label, random_visible_playlist,
};

use std::sync::{LazyLock, Mutex};

use qbz_models::Track;

/// The loaded favorite tracks as a play-ready queue source (Play all /
/// Shuffle). Set on the UI thread by `apply::tracks::apply_tracks`.
pub(crate) static FAV_CURRENT: LazyLock<Mutex<Vec<Track>>> = LazyLock::new(|| Mutex::new(Vec::new()));

/// The loaded favorite tracks as a play-ready queue (Play all).
pub fn play_tracks() -> Vec<Track> {
    FAV_CURRENT.lock().map(|c| c.clone()).unwrap_or_default()
}

/// The favorite tracks in a fresh random order (Shuffle). Mirrors
/// playlist::shuffled_tracks (time-seeded xorshift Fisher-Yates).
pub fn shuffled_tracks() -> Vec<Track> {
    let mut tracks = play_tracks();
    let mut seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(1)
        | 1;
    for i in (1..tracks.len()).rev() {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        let j = (seed % (i as u64 + 1)) as usize;
        tracks.swap(i, j);
    }
    tracks
}
