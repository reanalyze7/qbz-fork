//! L1 Memory Cache - In-memory LRU cache for audio data
//!
//! Fast access cache with configurable size limit and LRU eviction.
//! Evicted tracks can optionally spill to L2 disk cache.

mod core;
mod fetch_tracking;
mod stats;

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::PlaybackCache;

pub use stats::CacheStats;

/// Cached audio data for a track
#[derive(Clone)]
pub struct CachedTrack {
    pub track_id: u64,
    pub data: Vec<u8>,
    pub size_bytes: usize,
}

/// Internal cache state - all in one struct to avoid deadlocks
struct CacheState {
    /// Cached tracks keyed by track ID
    tracks: HashMap<u64, CachedTrack>,
    /// Order of access for LRU eviction (most recent at back)
    access_order: Vec<u64>,
    /// Current cache size in bytes
    current_size: usize,
    /// Track IDs currently being fetched
    fetching: HashSet<u64>,
    /// Track IDs whose last prefetch failed, with when it failed. Lets the
    /// prefetch scheduler back off a track that is currently un-fetchable
    /// (e.g. the account is being 403'd) instead of re-hammering it every
    /// queue tick and feeding a request storm (issue #637).
    failed: HashMap<u64, Instant>,
}

fn empty_state() -> CacheState {
    CacheState {
        tracks: HashMap::new(),
        access_order: Vec::new(),
        current_size: 0,
        fetching: HashSet::new(),
        failed: HashMap::new(),
    }
}

/// Audio cache manager with LRU eviction and optional disk spillover
///
/// Provides fast in-memory caching with automatic eviction when the
/// size limit is reached. Evicted tracks are written to the L2 disk
/// cache (if configured) for later retrieval.
pub struct AudioCache {
    state: Mutex<CacheState>,
    /// Maximum cache size in bytes
    max_size_bytes: usize,
    /// Optional disk-based L2 cache for evicted tracks
    playback_cache: Option<Arc<PlaybackCache>>,
}

impl Default for AudioCache {
    fn default() -> Self {
        Self::new(400 * 1024 * 1024) // 400MB for ~4-5 Hi-Res tracks
    }
}

impl AudioCache {
    /// Create a new cache with specified max size in bytes
    pub fn new(max_size_bytes: usize) -> Self {
        Self {
            state: Mutex::new(empty_state()),
            max_size_bytes,
            playback_cache: None,
        }
    }

    /// Create cache with disk spillover enabled
    pub fn with_playback_cache(max_size_bytes: usize, playback_cache: Arc<PlaybackCache>) -> Self {
        Self {
            state: Mutex::new(empty_state()),
            max_size_bytes,
            playback_cache: Some(playback_cache),
        }
    }

    /// Set the playback cache for disk spillover
    pub fn set_playback_cache(&mut self, cache: Arc<PlaybackCache>) {
        self.playback_cache = Some(cache);
    }

    /// Get the playback cache reference
    pub fn get_playback_cache(&self) -> Option<&Arc<PlaybackCache>> {
        self.playback_cache.as_ref()
    }
}
