//! The LRU + disk-spillover heart of the cache: `get`/`contains`/`insert`/
//! `clear`.

use super::{AudioCache, CachedTrack};

impl AudioCache {
    /// Get a track from cache if available
    pub fn get(&self, track_id: u64) -> Option<CachedTrack> {
        let mut state = self.state.lock().unwrap();

        let track = state.tracks.get(&track_id).cloned();

        if track.is_some() {
            // Update access order (move to back = most recently used)
            state.access_order.retain(|&id| id != track_id);
            state.access_order.push(track_id);
            log::debug!("Cache hit for track {}", track_id);
        } else {
            log::debug!("Cache miss for track {}", track_id);
        }

        track
    }

    /// Check if a track is in cache without updating access order
    pub fn contains(&self, track_id: u64) -> bool {
        self.state.lock().unwrap().tracks.contains_key(&track_id)
    }

    /// Insert a track into cache, evicting old entries to disk if needed
    pub fn insert(&self, track_id: u64, data: Vec<u8>) {
        let size = data.len();

        // Don't cache if track is larger than max cache size
        if size > self.max_size_bytes {
            log::warn!(
                "Track {} ({} bytes) too large for cache (max {} bytes)",
                track_id,
                size,
                self.max_size_bytes
            );
            return;
        }

        // Collect tracks to evict (to avoid holding lock while writing to disk)
        let mut tracks_to_spill: Vec<CachedTrack> = Vec::new();

        {
            let mut state = self.state.lock().unwrap();

            // Evict old entries to make room
            while state.current_size + size > self.max_size_bytes && !state.access_order.is_empty()
            {
                let oldest_id = state.access_order.remove(0);
                if let Some(track) = state.tracks.remove(&oldest_id) {
                    state.current_size = state.current_size.saturating_sub(track.size_bytes);
                    log::debug!(
                        "Evicting track {} ({} bytes) from memory cache",
                        oldest_id,
                        track.size_bytes
                    );
                    tracks_to_spill.push(track);
                }
            }
        }

        // Spill evicted tracks to disk cache (outside of lock)
        if let Some(playback_cache) = &self.playback_cache {
            for track in tracks_to_spill {
                playback_cache.insert(track.track_id, &track.data);
            }
        }

        let mut state = self.state.lock().unwrap();

        // If track already exists, update size tracking
        if let Some(existing) = state.tracks.get(&track_id) {
            state.current_size = state.current_size.saturating_sub(existing.size_bytes);
        }

        let cached = CachedTrack {
            track_id,
            data,
            size_bytes: size,
        };

        state.tracks.insert(track_id, cached);
        state.current_size += size;

        // Update access order
        state.access_order.retain(|&id| id != track_id);
        state.access_order.push(track_id);

        log::info!(
            "Cached track {} ({} bytes). Cache size: {}/{} bytes",
            track_id,
            size,
            state.current_size,
            self.max_size_bytes
        );
    }

    /// Clear all cached data (both L1 memory and L2 disk caches)
    pub fn clear(&self) {
        let mut state = self.state.lock().unwrap();
        state.tracks.clear();
        state.access_order.clear();
        state.current_size = 0;
        state.fetching.clear();
        state.failed.clear();
        log::info!("L1 memory cache cleared");

        // Also clear L2 disk cache if present
        if let Some(ref playback_cache) = self.playback_cache {
            playback_cache.clear();
            log::info!("L2 playback cache cleared");
        }
    }
}
