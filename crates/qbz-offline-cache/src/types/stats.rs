//! Plain data-only DTOs: aggregate stats, progress updates, and the
//! track-cache-initiation payload.

use serde::Serialize;

use super::OfflineCacheStatus;

/// Statistics about the offline cache
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OfflineCacheStats {
    pub total_tracks: usize,
    pub ready_tracks: usize,
    pub downloading_tracks: usize,
    pub failed_tracks: usize,
    pub total_size_bytes: u64,
    pub limit_bytes: Option<u64>,
    pub cache_path: String,
}

/// Progress update for caching a track
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheProgress {
    pub track_id: u64,
    pub progress_percent: u8,
    pub bytes_downloaded: u64,
    pub total_bytes: Option<u64>,
    pub status: OfflineCacheStatus,
}

/// Track metadata for initiating offline caching
#[derive(Debug, Clone)]
pub struct TrackCacheInfo {
    pub track_id: u64,
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub album_id: Option<String>,
    pub duration_secs: u64,
    pub quality: String,
    pub bit_depth: Option<u32>,
    pub sample_rate: Option<f64>,
}
