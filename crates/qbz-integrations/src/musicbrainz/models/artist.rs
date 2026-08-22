//! Artist search/lookup responses and supporting entity types

use serde::{Deserialize, Serialize};

use super::recording::ArtistRef;

/// Artist search response
#[derive(Debug, Deserialize)]
pub struct ArtistSearchResponse {
    pub created: Option<String>,
    pub count: i32,
    pub offset: i32,
    pub artists: Vec<ArtistResult>,
}

/// Single artist in search results
#[derive(Debug, Deserialize)]
pub struct ArtistResult {
    pub id: String,
    pub score: Option<i32>,
    pub name: String,
    #[serde(rename = "sort-name")]
    pub sort_name: Option<String>,
    #[serde(rename = "type")]
    pub artist_type: Option<String>,
    pub country: Option<String>,
    pub disambiguation: Option<String>,
    #[serde(default)]
    pub aliases: Option<Vec<Alias>>,
    #[serde(rename = "life-span")]
    pub life_span: Option<LifeSpan>,
    #[serde(default)]
    pub area: Option<Area>,
    #[serde(rename = "begin-area", default)]
    pub begin_area: Option<Area>,
    #[serde(default)]
    pub tags: Option<Vec<Tag>>,
}

/// Artist alias
#[derive(Debug, Deserialize)]
pub struct Alias {
    pub name: String,
    #[serde(rename = "sort-name")]
    pub sort_name: Option<String>,
    #[serde(rename = "type")]
    pub alias_type: Option<String>,
    pub locale: Option<String>,
    pub primary: Option<bool>,
}

/// MusicBrainz area (city, state, country, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Area {
    pub id: String,
    pub name: String,
    #[serde(rename = "sort-name")]
    pub sort_name: Option<String>,
    #[serde(rename = "type")]
    pub area_type: Option<String>,
}

/// Community tag (used for genres)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Tag {
    pub name: String,
    pub count: Option<i32>,
}

/// Life span for an artist
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LifeSpan {
    pub begin: Option<String>,
    pub end: Option<String>,
    pub ended: Option<bool>,
}

/// Relation between entities
#[derive(Debug, Deserialize)]
pub struct Relation {
    #[serde(rename = "type")]
    pub relation_type: String,
    #[serde(rename = "type-id")]
    pub type_id: Option<String>,
    pub direction: Option<String>,
    pub begin: Option<String>,
    pub end: Option<String>,
    pub ended: Option<bool>,
    pub attributes: Option<Vec<String>>,
    pub artist: Option<ArtistRef>,
}

/// Full artist response (with includes like relations, tags)
#[derive(Debug, Deserialize)]
pub struct ArtistFullResponse {
    pub id: String,
    pub name: String,
    #[serde(rename = "sort-name")]
    pub sort_name: Option<String>,
    #[serde(rename = "type")]
    pub artist_type: Option<String>,
    pub country: Option<String>,
    pub disambiguation: Option<String>,
    #[serde(rename = "life-span")]
    pub life_span: Option<LifeSpan>,
    #[serde(default)]
    pub area: Option<Area>,
    #[serde(rename = "begin-area", default)]
    pub begin_area: Option<Area>,
    pub relations: Option<Vec<Relation>>,
    #[serde(default)]
    pub tags: Option<Vec<Tag>>,
}

/// Artist browse response (from browse API)
#[derive(Debug, Deserialize)]
pub struct ArtistBrowseResponse {
    #[serde(rename = "artist-count")]
    pub artist_count: Option<i32>,
    #[serde(rename = "artist-offset")]
    pub artist_offset: Option<i32>,
    pub artists: Vec<ArtistResult>,
}
