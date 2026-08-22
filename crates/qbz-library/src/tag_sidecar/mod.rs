//! LocalLibrary album tag sidecar support.
//!
//! Sidecar files live next to album folders (default `.qbz.json`) and store
//! album-level + per-track metadata overrides.

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

mod apply;
mod io;

pub use apply::*;
pub use io::*;

const SIDECAR_FILE_NAME: &str = ".qbz.json";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AlbumMetadataOverride {
    pub album_title: Option<String>,
    pub album_artist: Option<String>,
    pub year: Option<u32>,
    pub genre: Option<String>,
    pub catalog_number: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackMetadataOverride {
    pub file_path: String,
    pub cue_start_secs: Option<f64>,
    pub title: Option<String>,
    pub disc_number: Option<u32>,
    pub track_number: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumTagSidecar {
    pub version: u32,
    pub updated_at: i64,
    pub album: AlbumMetadataOverride,
    pub tracks: Vec<TrackMetadataOverride>,
}

impl AlbumTagSidecar {
    pub fn new(album: AlbumMetadataOverride, tracks: Vec<TrackMetadataOverride>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        Self {
            version: 1,
            updated_at: now,
            album,
            tracks,
        }
    }
}
