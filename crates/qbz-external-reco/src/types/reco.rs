//! Resolved rows (validated to Qobuz).

use serde::{Deserialize, Serialize};

use super::RecoSource;

/// A resolved artist row (validated to a Qobuz artist).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistReco {
    pub qobuz_artist_id: u64,
    pub name: String,
    pub image_url: String,
    /// "Similar to X, Y, Z".
    #[serde(default)]
    pub subtitle: String,
    pub source: RecoSource,
}

/// A resolved album row (validated to a Qobuz album).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumReco {
    pub qobuz_album_id: String,
    pub title: String,
    pub artist: String,
    pub artist_id: String,
    pub year: String,
    pub quality_tier: String,
    pub quality_label: String,
    pub artwork_url: String,
    #[serde(default)]
    pub subtitle: String,
    #[serde(default = "default_source")]
    pub source: RecoSource,
}

fn default_source() -> RecoSource {
    RecoSource::Editorial
}

/// A resolved track row (validated to / sourced from a Qobuz track).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackReco {
    pub qobuz_track_id: u64,
    pub title: String,
    pub artist: String,
    pub artwork_url: String,
    pub source: RecoSource,
}
