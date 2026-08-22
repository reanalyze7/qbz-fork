//! Musician resolution and album-appearance types

use serde::{Deserialize, Serialize};

use super::confidence::MusicianConfidence;

/// Resolved musician with confidence assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedMusician {
    pub name: String,
    pub role: String,
    pub mbid: Option<String>,
    pub qobuz_artist_id: Option<i64>,
    pub confidence: MusicianConfidence,
    pub bands: Vec<String>,
    pub appears_on_count: usize,
}

impl ResolvedMusician {
    pub fn empty(name: String, role: String) -> Self {
        Self {
            name,
            role,
            mbid: None,
            qobuz_artist_id: None,
            confidence: MusicianConfidence::None,
            bands: Vec::new(),
            appears_on_count: 0,
        }
    }
}

/// Album appearance for a musician
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumAppearance {
    pub album_id: String,
    pub album_title: String,
    pub album_artwork: String,
    pub artist_name: String,
    pub year: Option<String>,
    pub role_on_album: String,
}

/// Musician appearances response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicianAppearances {
    pub albums: Vec<AlbumAppearance>,
    pub total: usize,
}
