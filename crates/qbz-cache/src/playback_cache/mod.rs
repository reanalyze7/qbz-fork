//! L2 Disk Cache - File-based playback cache
//!
//! Secondary cache for audio data evicted from memory.
//! Provides faster access than re-downloading from network.

mod access;
mod eviction;
mod init;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::SystemTime;

/// Entry metadata for tracking cache usage
#[derive(Debug, Clone)]
struct CacheEntry {
    #[allow(dead_code)]
    track_id: u64,
    size_bytes: u64,
    last_accessed: SystemTime,
}

/// Disk-based playback cache state
struct PlaybackCacheState {
    /// Track metadata keyed by track ID
    entries: HashMap<u64, CacheEntry>,
    /// Current total size in bytes
    current_size: u64,
}

/// Disk-based playback cache for evicted tracks
///
/// Stores audio data as files on disk with LRU eviction.
/// Files are named `{track_id}.audio` in the cache directory.
pub struct PlaybackCache {
    state: Mutex<PlaybackCacheState>,
    /// Cache directory path
    cache_dir: PathBuf,
    /// Maximum cache size in bytes
    max_size_bytes: u64,
}

/// Playback cache statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct PlaybackCacheStats {
    pub cached_tracks: usize,
    pub current_size_bytes: u64,
    pub max_size_bytes: u64,
}
