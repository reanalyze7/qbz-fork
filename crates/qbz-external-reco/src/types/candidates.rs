//! Raw candidates (pre Qobuz validation).

use super::RecoSource;

#[derive(Debug, Clone)]
pub struct ArtistCandidate {
    pub name: String,
    pub source: RecoSource,
    pub score: f32,
    /// "Similar to X, Y, Z" line, built from the seeds that surfaced this artist.
    pub subtitle: String,
}

#[derive(Debug, Clone)]
pub struct AlbumCandidate {
    pub artist: String,
    pub title: String,
    pub upc: Option<String>,
    pub source: RecoSource,
    pub score: f32,
    /// "Similar to …" / "You've scrobbled {artist} before" line.
    pub subtitle: String,
}

#[derive(Debug, Clone)]
pub struct TrackCandidate {
    pub artist: String,
    pub title: String,
    pub album: Option<String>,
    pub duration_ms: Option<u64>,
    pub isrc: Option<String>,
    pub recording_mbid: Option<String>,
    pub source: RecoSource,
    pub score: f32,
}
