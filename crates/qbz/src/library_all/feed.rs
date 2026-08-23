//! The plain, `Send` feed item + its recency-rank helper.

/// Plain, `Send` feed item produced on the worker thread.
#[derive(Clone, Default)]
pub struct Feed {
    pub kind: String,   // track | album | artist | playlist | label
    pub group: String,  // favorites | following | purchases
    pub source: String, // qobuz | local
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub artist: String,
    pub artist_id: String,
    pub album: String,
    pub album_id: String,
    pub image_url: String,
    pub quality_tier: String,
    pub quality_detail: String,
    pub is_favorite: bool,
    /// Genre name (albums + tracks carry one; artists/labels/playlists ""). Feeds
    /// the client-side genre filter — "" is excluded when a genre is selected.
    pub genre: String,
    /// Playlist ownership (only meaningful for kind == "playlist"): owned →
    /// favorite affordance; foreign Qobuz → follow + copy.
    pub playlist_owned: bool,
    pub playlist_following: bool,
    pub playlist_copied: bool,
    /// Recency proxy in [0.0, 1.0]; 0.0 = most-recently added. Each source list
    /// comes back date-desc, so `index / len` interleaves the sources by recency
    /// without needing exact per-item timestamps.
    pub added_rank: f32,
}

pub(super) fn rank(i: usize, n: usize) -> f32 {
    if n <= 1 {
        0.0
    } else {
        i as f32 / n as f32
    }
}
