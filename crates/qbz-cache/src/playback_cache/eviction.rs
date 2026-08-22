//! LRU eviction + introspection: `evict_if_needed`, `clear`, `stats`,
//! `cache_dir`.

use std::fs;
use std::path::PathBuf;

use super::{PlaybackCache, PlaybackCacheStats};

impl PlaybackCache {
    /// Evict oldest entries to make room for new data
    pub(super) fn evict_if_needed(&self, needed_bytes: u64) {
        let mut state = self.state.lock().unwrap();

        while state.current_size + needed_bytes > self.max_size_bytes && !state.entries.is_empty() {
            // Find oldest entry
            let oldest_id = state
                .entries
                .iter()
                .min_by_key(|(_, e)| e.last_accessed)
                .map(|(id, _)| *id);

            if let Some(track_id) = oldest_id {
                if let Some(entry) = state.entries.remove(&track_id) {
                    state.current_size = state.current_size.saturating_sub(entry.size_bytes);

                    // Delete file
                    let path = self.track_path(track_id);
                    if let Err(e) = fs::remove_file(&path) {
                        log::debug!("Failed to delete playback cache file: {}", e);
                    } else {
                        log::debug!(
                            "Evicted track {} from playback cache ({} KB)",
                            track_id,
                            entry.size_bytes / 1024
                        );
                    }
                }
            } else {
                break;
            }
        }
    }

    /// Clear the entire cache
    pub fn clear(&self) {
        let mut state = self.state.lock().unwrap();

        for track_id in state.entries.keys() {
            let path = self.cache_dir.join(format!("{}.audio", track_id));
            let _ = fs::remove_file(&path);
        }

        state.entries.clear();
        state.current_size = 0;

        log::info!("Playback cache cleared");
    }

    /// Get cache statistics
    pub fn stats(&self) -> PlaybackCacheStats {
        let state = self.state.lock().unwrap();
        PlaybackCacheStats {
            cached_tracks: state.entries.len(),
            current_size_bytes: state.current_size,
            max_size_bytes: self.max_size_bytes,
        }
    }

    /// Get the cache directory path
    pub fn cache_dir(&self) -> &PathBuf {
        &self.cache_dir
    }
}
