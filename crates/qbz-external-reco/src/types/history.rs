//! Listening-history snapshot types.

use std::collections::HashSet;

/// Local listening signal from the per-user `reco_events` store (Qobuz ids).
#[derive(Debug, Clone, Default)]
pub struct LocalHistory {
    /// Artists the user already knows (played > threshold or favorited).
    pub known_artist_ids: HashSet<u64>,
    /// Tracks played in-app.
    pub played_track_ids: HashSet<u64>,
    /// Albums played in-app (the local "already heard albums" set).
    pub played_album_ids: HashSet<String>,
}

/// External listening signal (normalized) for the "not heard / not scrobbled"
/// filters. Gathered ONCE per build and shared across the per-row builders.
#[derive(Debug, Clone, Default)]
pub struct ExtHistory {
    /// Normalized artist names the user has listened to (Last.fm + LB).
    pub artist_names: HashSet<String>,
    /// Normalized "artist|title" track keys (scrobbled set).
    pub track_keys: HashSet<String>,
    /// Normalized "artist|album" keys (scrobbled-album set).
    pub album_keys: HashSet<String>,
}
