//! Album and artist aggregation models

use super::audio_format::AudioFormat;
use serde::{Deserialize, Serialize};

/// One page of metadata-grouped local albums plus the total count of
/// albums matching the same filter (for scrollbar pre-allocation on the
/// frontend). Returned by `Database::get_albums_metadata_page`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumsMetadataPage {
    pub albums: Vec<LocalAlbum>,
    pub total: u64,
}

/// An album aggregated from local tracks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalAlbum {
    pub id: String,
    pub title: String,
    pub artist: String,
    /// All contributing artists (comma-separated) - used for matching in artist view
    /// This includes all unique album_artist/artist values from the album's tracks
    #[serde(default)]
    pub all_artists: String,
    pub year: Option<u32>,
    pub catalog_number: Option<String>,
    pub artwork_path: Option<String>,
    pub track_count: u32,
    pub total_duration_secs: u64,
    pub format: AudioFormat,
    pub bit_depth: Option<u32>,
    pub sample_rate: f64, // Changed from u32 to f64 for decimal precision
    pub directory_path: String,
    /// Comma-separated list of distinct folder keys that contributed
    /// tracks to this album. Populated only by the metadata-grouped
    /// Albums query (`get_albums_metadata_grouped`); `None` for folder-
    /// grouped rows. The frontend uses this to render a tooltip when N
    /// folders > 1.
    #[serde(default)]
    pub source_folders: Option<String>,
    /// Source of the album: "user" for local files, "qobuz_download" for offline cached
    #[serde(default = "default_source")]
    pub source: String,
}

fn default_source() -> String {
    "user".to_string()
}

/// An artist aggregated from local tracks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalArtist {
    pub name: String,
    pub album_count: u32,
    pub track_count: u32,
}
