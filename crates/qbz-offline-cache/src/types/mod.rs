//! Shared DTOs for the offline cache (moved verbatim from
//! `src-tauri/src/offline_cache/mod.rs`). Pure serde — no Tauri, no I/O.
//!
//! Split into `track_info` (`CachedTrackInfo` + its cover-path resolution
//! logic, and `ReadyTrackForSync`) and `stats` (the plain data-only
//! structs); the status enum stays here since it's the one piece of logic
//! shared across every other DTO in this module.

mod stats;
mod track_info;

pub use stats::{CacheProgress, OfflineCacheStats, TrackCacheInfo};
pub use track_info::{CachedTrackInfo, ReadyTrackForSync};

use serde::{Deserialize, Serialize};

/// Cache status for a track in offline storage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OfflineCacheStatus {
    Queued,
    Downloading,
    Ready,
    Failed,
}

impl OfflineCacheStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Downloading => "downloading",
            Self::Ready => "ready",
            Self::Failed => "failed",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "queued" => Self::Queued,
            "downloading" => Self::Downloading,
            "ready" => Self::Ready,
            "failed" => Self::Failed,
            _ => Self::Failed,
        }
    }
}
