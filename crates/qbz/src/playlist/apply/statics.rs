//! The `CURRENT` Qobuz-track cache + the `MIXED` sidecar flag, and the pure
//! readers built on top of them.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LazyLock, Mutex};

use qbz_models::Track;

/// The currently-loaded playlist's QOBUZ tracks (server order), for
/// play-all / per-track play of pure-Qobuz details AND for resolving a
/// catalog id to its `playlist_track_id` (removal) — the full `Track`
/// keeps what the `TrackItem` row model drops.
pub(in crate::playlist) static CURRENT: LazyLock<Mutex<Vec<Track>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

/// True while the open ONLINE Qobuz detail carries sidecar (local)
/// rows — the play/shuffle/per-row-play paths then route through
/// `local_playlist`'s merged queue snapshot instead of the Qobuz-only
/// `CURRENT` cache. Set in `apply`, cleared in `reset`.
pub(super) static MIXED: AtomicBool = AtomicBool::new(false);

/// Whether the open ONLINE Qobuz detail is a mixed ("carrete") playlist.
pub fn is_mixed() -> bool {
    MIXED.load(Ordering::Relaxed)
}

pub fn current_tracks() -> Vec<Track> {
    CURRENT.lock().map(|c| c.clone()).unwrap_or_default()
}

/// The current playlist tracks in a fresh random order (Shuffle).
pub fn shuffled_tracks() -> Vec<Track> {
    let mut tracks = current_tracks();
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
