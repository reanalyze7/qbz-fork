//! Cache introspection: [`CacheStats`] and [`AudioCache::stats`].

use super::AudioCache;

/// Cache statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct CacheStats {
    pub cached_tracks: usize,
    pub current_size_bytes: usize,
    pub max_size_bytes: usize,
    pub fetching_count: usize,
}

impl AudioCache {
    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let state = self.state.lock().unwrap();
        CacheStats {
            cached_tracks: state.tracks.len(),
            current_size_bytes: state.current_size,
            max_size_bytes: self.max_size_bytes,
            fetching_count: state.fetching.len(),
        }
    }
}
