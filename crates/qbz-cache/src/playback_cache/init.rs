//! Construction and the on-disk-scan-to-rebuild-index logic.

use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

use super::{CacheEntry, PlaybackCache, PlaybackCacheState};

impl PlaybackCache {
    /// Create a new playback cache with default location
    ///
    /// Default path: `~/.cache/qbz/playback/`
    pub fn new(max_size_bytes: u64) -> Result<Self, String> {
        let cache_dir = dirs::cache_dir()
            .ok_or("Could not determine cache directory")?
            .join("qbz")
            .join("playback");

        Self::with_path(cache_dir, max_size_bytes)
    }

    /// Create a new playback cache at a specific path
    pub fn with_path(cache_dir: PathBuf, max_size_bytes: u64) -> Result<Self, String> {
        // Create directory
        fs::create_dir_all(&cache_dir)
            .map_err(|e| format!("Failed to create playback cache directory: {}", e))?;

        let cache = Self {
            state: std::sync::Mutex::new(PlaybackCacheState {
                entries: std::collections::HashMap::new(),
                current_size: 0,
            }),
            cache_dir,
            max_size_bytes,
        };

        // Scan existing files to rebuild state
        cache.rebuild_state();

        log::info!(
            "Playback cache initialized at {:?} (max {} MB)",
            cache.cache_dir,
            max_size_bytes / (1024 * 1024)
        );

        Ok(cache)
    }

    /// Rebuild cache state from existing files on disk
    fn rebuild_state(&self) {
        let mut state = self.state.lock().unwrap();
        state.entries.clear();
        state.current_size = 0;

        if let Ok(entries) = fs::read_dir(&self.cache_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        // Parse track ID from filename (format: {track_id}.audio)
                        if let Some(filename) = entry.file_name().to_str() {
                            if let Some(id_str) = filename.strip_suffix(".audio") {
                                if let Ok(track_id) = id_str.parse::<u64>() {
                                    let size = metadata.len();
                                    let last_accessed =
                                        metadata.accessed().unwrap_or_else(|_| SystemTime::now());

                                    state.entries.insert(
                                        track_id,
                                        CacheEntry {
                                            track_id,
                                            size_bytes: size,
                                            last_accessed,
                                        },
                                    );
                                    state.current_size += size;
                                }
                            }
                        }
                    }
                }
            }
        }

        log::info!(
            "Playback cache rebuilt: {} tracks, {} MB",
            state.entries.len(),
            state.current_size / (1024 * 1024)
        );
    }

    /// Get file path for a track
    pub(super) fn track_path(&self, track_id: u64) -> PathBuf {
        self.cache_dir.join(format!("{}.audio", track_id))
    }
}
