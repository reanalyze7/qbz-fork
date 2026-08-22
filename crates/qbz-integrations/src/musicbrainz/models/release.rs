//! Release search/lookup responses and their supporting entity types

use serde::Deserialize;

use super::artist::Tag;
use super::recording::{ArtistCredit, ReleaseGroupRef};

/// Release search response
#[derive(Debug, Deserialize)]
pub struct ReleaseSearchResponse {
    pub created: Option<String>,
    pub count: i32,
    pub offset: i32,
    pub releases: Vec<ReleaseResult>,
}

/// Single release in search results
#[derive(Debug, Deserialize)]
pub struct ReleaseResult {
    pub id: String,
    pub score: Option<i32>,
    pub title: String,
    pub status: Option<String>,
    pub date: Option<String>,
    pub country: Option<String>,
    pub barcode: Option<String>,
    #[serde(rename = "label-info")]
    pub label_info: Option<Vec<LabelInfo>>,
    #[serde(rename = "release-group")]
    pub release_group: Option<ReleaseGroupRef>,
    #[serde(rename = "artist-credit")]
    pub artist_credit: Option<Vec<ArtistCredit>>,
    pub media: Option<Vec<ReleaseSearchMedium>>,
    #[serde(rename = "track-count")]
    pub track_count: Option<u16>,
}

/// Simplified medium info for search results
#[derive(Debug, Deserialize)]
pub struct ReleaseSearchMedium {
    pub format: Option<String>,
    #[serde(rename = "track-count")]
    pub track_count: Option<u16>,
}

/// Label information
#[derive(Debug, Deserialize)]
pub struct LabelInfo {
    #[serde(rename = "catalog-number")]
    pub catalog_number: Option<String>,
    pub label: Option<LabelRef>,
}

/// Reference to a label
#[derive(Debug, Deserialize)]
pub struct LabelRef {
    pub id: String,
    pub name: String,
}

/// Full release response from lookup endpoint (with tracks)
#[derive(Debug, Deserialize)]
pub struct ReleaseFullResponse {
    pub id: String,
    pub title: String,
    pub status: Option<String>,
    pub date: Option<String>,
    pub country: Option<String>,
    pub barcode: Option<String>,
    #[serde(rename = "artist-credit")]
    pub artist_credit: Option<Vec<ArtistCredit>>,
    #[serde(rename = "label-info")]
    pub label_info: Option<Vec<LabelInfo>>,
    #[serde(rename = "release-group")]
    pub release_group: Option<ReleaseGroupRef>,
    pub media: Option<Vec<Medium>>,
    pub tags: Option<Vec<Tag>>,
}

/// A medium (disc) containing tracks
#[derive(Debug, Deserialize)]
pub struct Medium {
    pub position: Option<u8>,
    pub format: Option<String>,
    #[serde(rename = "track-count")]
    pub track_count: Option<u16>,
    pub tracks: Option<Vec<MediumTrack>>,
}

/// A track on a medium
#[derive(Debug, Deserialize)]
pub struct MediumTrack {
    pub position: Option<u8>,
    pub number: Option<String>,
    pub title: Option<String>,
    pub length: Option<i64>,
    pub recording: Option<TrackRecording>,
}

/// Recording reference within a track
#[derive(Debug, Deserialize)]
pub struct TrackRecording {
    pub id: String,
    pub title: Option<String>,
    pub length: Option<i64>,
    #[serde(rename = "artist-credit")]
    pub artist_credit: Option<Vec<ArtistCredit>>,
}
