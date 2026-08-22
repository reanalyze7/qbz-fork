//! Recording search/lookup responses and their supporting reference types

use serde::Deserialize;

/// Recording search response
#[derive(Debug, Deserialize)]
pub struct RecordingSearchResponse {
    pub created: Option<String>,
    pub count: i32,
    pub offset: i32,
    pub recordings: Vec<RecordingResult>,
}

/// Single recording in search results
#[derive(Debug, Deserialize)]
pub struct RecordingResult {
    pub id: String,
    pub score: Option<i32>,
    pub title: Option<String>,
    pub length: Option<i64>,
    #[serde(rename = "artist-credit")]
    pub artist_credit: Option<Vec<ArtistCredit>>,
    pub isrcs: Option<Vec<String>>,
    pub releases: Option<Vec<ReleaseRef>>,
}

/// Response from the single-recording lookup `/recording/{mbid}?inc=isrcs`.
#[derive(Debug, Deserialize)]
pub struct RecordingLookupResponse {
    pub id: String,
    pub title: Option<String>,
    pub isrcs: Option<Vec<String>>,
}

/// Artist credit entry
#[derive(Debug, Deserialize)]
pub struct ArtistCredit {
    pub name: Option<String>,
    pub joinphrase: Option<String>,
    pub artist: ArtistRef,
}

/// Reference to an artist
#[derive(Debug, Deserialize)]
pub struct ArtistRef {
    pub id: String,
    pub name: String,
    #[serde(rename = "sort-name")]
    pub sort_name: Option<String>,
    pub disambiguation: Option<String>,
}

/// Reference to a release (album)
#[derive(Debug, Deserialize)]
pub struct ReleaseRef {
    pub id: String,
    pub title: Option<String>,
    pub status: Option<String>,
    pub date: Option<String>,
    pub country: Option<String>,
    #[serde(rename = "release-group")]
    pub release_group: Option<ReleaseGroupRef>,
}

/// Reference to a release group
#[derive(Debug, Deserialize)]
pub struct ReleaseGroupRef {
    pub id: String,
    #[serde(rename = "primary-type")]
    pub primary_type: Option<String>,
}
