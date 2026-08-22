use serde::{Deserialize, Serialize};

use super::provider::RemoteProvider;

/// Lightweight search result for displaying in results list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteAlbumSearchResult {
    /// Provider identifier
    pub provider: RemoteProvider,
    /// Provider-specific ID (MusicBrainz release ID or Discogs release ID)
    pub provider_id: String,
    /// Album title
    pub title: String,
    /// Album artist
    pub artist: String,
    /// Release year (extracted from date)
    pub year: Option<u16>,
    /// Number of tracks (if available from search)
    pub track_count: Option<u16>,
    /// Release country
    pub country: Option<String>,
    /// Record label
    pub label: Option<String>,
    /// Catalog number
    pub catalog_number: Option<String>,
    /// Match confidence (0-100, provider-specific)
    pub confidence: Option<u8>,
    /// Format info (e.g., "CD", "Vinyl", "Digital")
    pub format: Option<String>,
}

/// Full album metadata with tracks (for applying to form)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteAlbumMetadata {
    /// Provider identifier
    pub provider: RemoteProvider,
    /// Provider-specific ID
    pub provider_id: String,
    /// Album title
    pub title: String,
    /// Album artist
    pub artist: String,
    /// Release year
    pub year: Option<u16>,
    /// Genres/styles/tags
    pub genres: Vec<String>,
    /// Record label
    pub label: Option<String>,
    /// Catalog number
    pub catalog_number: Option<String>,
    /// Release country
    pub country: Option<String>,
    /// Barcode/UPC (if available)
    pub barcode: Option<String>,
    /// Track list organized by disc
    pub tracks: Vec<RemoteTrackMetadata>,
    /// Total disc count
    pub disc_count: u8,
    /// URL to view on provider website
    pub source_url: Option<String>,
}

/// Single track metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteTrackMetadata {
    /// Disc number (1-based)
    pub disc_number: u8,
    /// Track number within disc (1-based)
    pub track_number: u8,
    /// Track title
    pub title: String,
    /// Duration in milliseconds (if available)
    pub duration_ms: Option<u32>,
}
