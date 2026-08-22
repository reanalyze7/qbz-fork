//! Public config/result/track types for the suggestions engine.

use serde::{Deserialize, Serialize};

/// Configuration for suggestion generation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SuggestionConfig {
    /// Maximum number of artists to consider for suggestions
    pub max_artists: usize,
    /// Number of tracks to fetch per artist
    pub tracks_per_artist: usize,
    /// Maximum total tracks in the suggestion pool
    pub max_pool_size: usize,
    /// Maximum age (days) for vector freshness
    pub vector_max_age_days: i64,
    /// Minimum similarity score to include artist
    pub min_similarity: f32,
    /// Skip building vectors - only use existing cached vectors (faster but may have fewer results)
    pub skip_vector_build: bool,
}

impl Default for SuggestionConfig {
    fn default() -> Self {
        Self {
            max_artists: 30,      // Increased from 20 for more variety
            tracks_per_artist: 6, // Increased from 5
            max_pool_size: 150,   // Increased from 100
            vector_max_age_days: 7,
            min_similarity: 0.1,
            skip_vector_build: false,
        }
    }
}

/// A suggested track with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedTrack {
    /// Qobuz track ID
    pub track_id: u64,
    /// Track title
    pub title: String,
    /// Artist name
    pub artist_name: String,
    /// Artist Qobuz ID (for navigation)
    pub artist_id: Option<u64>,
    /// Artist MBID (if known)
    pub artist_mbid: Option<String>,
    /// Album title
    pub album_title: String,
    /// Album ID for cover art
    pub album_id: String,
    /// Direct URL to album cover image
    pub album_image_url: Option<String>,
    /// Duration in seconds
    pub duration: u32,
    /// Similarity score (higher = more similar to playlist)
    pub similarity_score: f32,
    /// Reason for suggestion (for dev mode)
    pub reason: Option<String>,
}

/// Result of suggestion generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionResult {
    /// Pool of suggested tracks
    pub tracks: Vec<SuggestedTrack>,
    /// Artists that contributed to suggestions
    pub source_artists: Vec<String>,
    /// Number of playlist artists analyzed
    pub playlist_artists_count: usize,
    /// Number of similar artists found
    pub similar_artists_count: usize,
}
