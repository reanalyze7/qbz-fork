//! Artist metadata and discovery pipeline response types

use serde::{Deserialize, Serialize};

use super::artist::LifeSpan;
use super::confidence::ArtistType;

/// Precision level for artist location data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LocationPrecision {
    City,
    State,
    Country,
}

/// Resolved location for an artist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistLocation {
    pub city: Option<String>,
    pub area_id: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub display_name: String,
    pub precision: LocationPrecision,
}

/// Affinity seeds extracted from an artist's tags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffinitySeeds {
    pub genres: Vec<String>,
    pub tags: Vec<String>,
    pub normalized_seeds: Vec<String>,
}

/// Complete artist metadata for location discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistMetadata {
    pub mbid: String,
    pub name: String,
    pub artist_type: ArtistType,
    pub life_span: Option<LifeSpan>,
    pub location: Option<ArtistLocation>,
    pub affinity_seeds: AffinitySeeds,
}

/// A candidate artist from location discovery, validated against Qobuz
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationCandidate {
    pub mbid: String,
    pub mb_name: String,
    pub qobuz_id: Option<i64>,
    pub qobuz_name: Option<String>,
    pub qobuz_image: Option<String>,
    pub score: i32,
    pub genres: Vec<String>,
    pub qobuz_albums_count: Option<u32>,
}

/// Response from the location discovery pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationDiscoveryResponse {
    pub artists: Vec<LocationCandidate>,
    pub scene_label: String,
    pub genre_summary: String,
    pub total_candidates: usize,
    pub has_more: bool,
    pub next_offset: usize,
}

/// One candidate from the "you may also like" tag-based discovery pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryArtist {
    pub mbid: String,
    pub name: String,
    pub qobuz_id: Option<u64>,
}

/// Result of the tag-based discovery pipeline. `primary_tag` is the tag
/// the discovery was seeded with — frontends save dismissals keyed by
/// it so a "thumbs down" stays sticky for that tag across artists.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryResponse {
    pub artists: Vec<DiscoveryArtist>,
    pub primary_tag: String,
}
