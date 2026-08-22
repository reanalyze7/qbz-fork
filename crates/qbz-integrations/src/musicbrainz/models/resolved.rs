//! Resolved types used for caching/output after matching against Qobuz

use serde::{Deserialize, Serialize};

use super::confidence::{ArtistType, MatchConfidence};

/// Resolved artist with all metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedArtist {
    pub mbid: String,
    pub name: String,
    pub sort_name: Option<String>,
    pub artist_type: ArtistType,
    pub country: Option<String>,
    pub disambiguation: Option<String>,
    pub confidence: MatchConfidence,
}

/// Resolved track with MusicBrainz data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedTrack {
    pub recording_mbid: String,
    pub title: String,
    pub artist_mbids: Vec<String>,
    pub release_mbid: Option<String>,
    pub isrcs: Vec<String>,
    pub confidence: MatchConfidence,
}

/// Resolved release (album) with MusicBrainz data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedRelease {
    pub mbid: String,
    pub title: String,
    pub release_group_mbid: Option<String>,
    pub date: Option<String>,
    pub country: Option<String>,
    pub confidence: MatchConfidence,
}
