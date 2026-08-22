//! Discogs DTOs: search results, artwork image options, and full release metadata.

use serde::Deserialize;

/// Search result from Discogs API
#[derive(Debug, Deserialize, serde::Serialize, Clone)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
}

#[derive(Debug, Deserialize, serde::Serialize, Clone)]
pub struct SearchResult {
    pub id: u64,
    pub cover_image: Option<String>,
    pub thumb: Option<String>,
    pub title: String,
    #[serde(rename = "type")]
    pub result_type: String,
}

/// Image option for artwork selection
#[derive(Debug, Deserialize, serde::Serialize, Clone)]
pub struct DiscogsImageOption {
    pub url: String,
    pub width: u32,
    pub height: u32,
    #[serde(rename = "type")]
    pub image_type: String,
    pub release_title: Option<String>,
    pub release_year: Option<u32>,
}

/// Release details from Discogs API (internal, for artwork)
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub(super) struct ReleaseDetails {
    pub(super) id: u64,
    pub(super) title: String,
    pub(super) year: Option<u32>,
    pub(super) images: Option<Vec<ReleaseImage>>,
}

/// Image from release details
#[derive(Debug, Deserialize)]
pub(super) struct ReleaseImage {
    pub(super) uri: String,
    pub(super) width: u32,
    pub(super) height: u32,
    #[serde(rename = "type")]
    pub(super) image_type: String,
}

// ============ Public Metadata Structures ============

/// Full release metadata from Discogs (for tag editor)
#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct DiscogsReleaseMetadata {
    pub id: u64,
    pub title: String,
    pub artists: Option<Vec<DiscogsArtist>>,
    pub year: Option<u32>,
    pub genres: Option<Vec<String>>,
    pub styles: Option<Vec<String>>,
    pub labels: Option<Vec<DiscogsLabel>>,
    pub tracklist: Option<Vec<DiscogsTrack>>,
    pub country: Option<String>,
    /// URL to view on Discogs
    pub uri: Option<String>,
}

/// Artist in Discogs release
#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct DiscogsArtist {
    pub name: String,
    pub id: Option<u64>,
    /// Join phrase (e.g., " & ", " feat. ")
    pub join: Option<String>,
}

/// Label in Discogs release
#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct DiscogsLabel {
    pub name: String,
    pub catno: Option<String>,
    pub id: Option<u64>,
}

/// Track in Discogs release
#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct DiscogsTrack {
    /// Position (e.g., "1", "A1", "1-1")
    pub position: String,
    pub title: String,
    /// Duration as string (e.g., "3:45")
    pub duration: Option<String>,
    /// Track type (e.g., "track", "heading")
    #[serde(rename = "type_")]
    pub track_type: Option<String>,
}

/// Extended search result with more metadata
#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct DiscogsSearchResultExtended {
    pub id: u64,
    pub title: String,
    #[serde(rename = "type")]
    pub result_type: String,
    pub year: Option<String>,
    pub country: Option<String>,
    pub label: Option<Vec<String>>,
    pub catno: Option<String>,
    pub format: Option<Vec<String>>,
    pub cover_image: Option<String>,
}
