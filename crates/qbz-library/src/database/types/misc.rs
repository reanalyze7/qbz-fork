//! Small standalone value types used across several `database` submodules.

#[derive(Debug, Clone)]
pub struct AlbumTrackUpdate {
    pub id: i64,
    pub title: String,
    pub disc_number: Option<u32>,
    pub track_number: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct TrackMetadataUpdateFull {
    pub id: i64,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub album_artist: Option<String>,
    pub album_group_title: String,
    pub track_number: Option<u32>,
    pub disc_number: Option<u32>,
    pub year: Option<u32>,
    pub genre: Option<String>,
    pub catalog_number: Option<String>,
}

/// Library statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct LibraryStats {
    pub track_count: u32,
    pub album_count: u32,
    pub artist_count: u32,
    pub total_duration_secs: u64,
    pub total_size_bytes: u64,
}

/// Library folder with metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryFolder {
    pub id: i64,
    pub path: String,
    pub alias: Option<String>,
    pub enabled: bool,
    pub is_network: bool,
    pub network_fs_type: Option<String>,
    pub user_override_network: bool,
    pub last_scan: Option<i64>,
}
