//! Top-level `/artist/page` and `/artist/story` response types.

use serde::{Deserialize, Serialize};

use super::{PageArtistPlaylists, PageArtistRelease, PageArtistReleaseGroup, PageArtistTrack};

/// Top-level response from /artist/page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageArtistResponse {
    pub id: u64,
    pub name: PageArtistName,
    pub artist_category: Option<String>,
    pub biography: Option<PageArtistBiography>,
    pub images: Option<PageArtistImages>,
    pub similar_artists: Option<PageArtistSimilar>,
    pub top_tracks: Option<Vec<PageArtistTrack>>,
    pub last_release: Option<PageArtistRelease>,
    pub releases: Option<Vec<PageArtistReleaseGroup>>,
    pub tracks_appears_on: Option<Vec<PageArtistTrack>>,
    pub playlists: Option<PageArtistPlaylists>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageArtistName {
    pub display: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageArtistBiography {
    pub content: Option<String>,
    pub source: Option<serde_json::Value>,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageArtistImages {
    pub portrait: Option<PageArtistPortrait>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageArtistPortrait {
    pub hash: String,
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageArtistSimilar {
    pub has_more: bool,
    pub items: Vec<PageArtistSimilarItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageArtistSimilarItem {
    pub id: u64,
    pub name: PageArtistName,
    pub images: Option<PageArtistImages>,
}

/// Response from /artist/story (Magazine / editorial articles about the artist).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistStoryResponse {
    pub has_more: bool,
    #[serde(default)]
    pub items: Vec<ArtistStoryItem>,
}

/// A single Magazine story. `image`/`images[].url` are ready-to-use signed
/// arc-cdn URLs — do NOT run them through the portrait hash/format builder.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistStoryItem {
    pub id: String,
    pub title: String,
    /// Epoch SECONDS, not an ISO string.
    #[serde(default)]
    pub display_date: Option<i64>,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub images: Option<Vec<ArtistStoryImage>>,
    #[serde(default)]
    pub description_short: Option<String>,
    #[serde(default)]
    pub authors: Option<Vec<ArtistStoryAuthor>>,
    #[serde(default)]
    pub section_slugs: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistStoryImage {
    #[serde(default)]
    pub format: Option<String>,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistStoryAuthor {
    pub name: String,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub slug: Option<String>,
}
