//! The hot read/write path: `contains`, `get`, `insert`.

use std::fs;
use std::io::{Read, Write};
use std::time::SystemTime;

use super::{CacheEntry, PlaybackCache};

impl PlaybackCache {
    /// Check if a track is in the cache
    pub fn contains(&self, track_id: u64) -> bool {
        self.state.lock().unwrap().entries.contains_key(&track_id)
    }

    /// Get a track from the cache
    pub fn get(&self, track_id: u64) -> Option<Vec<u8>> {
        let path = self.track_path(track_id);

        // Check if file exists and read it
        if !path.exists() {
            // File was deleted externally, update state
            let mut state = self.state.lock().unwrap();
            if let Some(entry) = state.entries.remove(&track_id) {
                state.current_size = state.current_size.saturating_sub(entry.size_bytes);
            }
            return None;
        }

        match fs::File::open(&path) {
            Ok(mut file) => {
                let mut data = Vec::new();
                if file.read_to_end(&mut data).is_ok() {
                    // Update last accessed time
                    let mut state = self.state.lock().unwrap();
                    if let Some(entry) = state.entries.get_mut(&track_id) {
                        entry.last_accessed = SystemTime::now();
                    }

                    // Touch file to update filesystem access time
                    let _ = filetime::set_file_atime(&path, filetime::FileTime::now());

                    log::debug!(
                        "Playback cache hit for track {} ({} bytes)",
                        track_id,
                        data.len()
                    );
                    Some(data)
                } else {
                    log::warn!("Failed to read playback cache file for track {}", track_id);
                    None
                }
            }
            Err(e) => {
                log::warn!(
                    "Failed to open playback cache file for track {}: {}",
                    track_id,
                    e
                );
                None
            }
        }
    }

    /// Insert a track into the cache (called when evicting from memory cache)
    pub fn insert(&self, track_id: u64, data: &[u8]) {
        let size = data.len() as u64;

        // Don't cache if larger than max size
        if size > self.max_size_bytes {
            log::debug!(
                "Track {} too large for playback cache ({} MB > {} MB)",
                track_id,
                size / (1024 * 1024),
                self.max_size_bytes / (1024 * 1024)
            );
            return;
        }

        // Evict old entries if needed
        self.evict_if_needed(size);

        let path = self.track_path(track_id);

        // Write file
        match fs::File::create(&path) {
            Ok(mut file) => {
                if file.write_all(data).is_ok() {
                    let mut state = self.state.lock().unwrap();

                    // Remove old entry if exists
                    if let Some(old) = state.entries.remove(&track_id) {
                        state.current_size = state.current_size.saturating_sub(old.size_bytes);
                    }

                    // Add new entry
                    state.entries.insert(
                        track_id,
                        CacheEntry {
                            track_id,
                            size_bytes: size,
                            last_accessed: SystemTime::now(),
                        },
                    );
                    state.current_size += size;

                    log::info!(
                        "Saved track {} to playback cache ({} KB). Total: {} MB / {} MB",
                        track_id,
                        size / 1024,
                        state.current_size / (1024 * 1024),
                        self.max_size_bytes / (1024 * 1024)
                    );
                } else {
                    log::warn!("Failed to write playback cache file for track {}", track_id);
                    let _ = fs::remove_file(&path);
                }
            }
            Err(e) => {
                log::warn!(
                    "Failed to create playback cache file for track {}: {}",
                    track_id,
                    e
                );
            }
        }
    }
}
