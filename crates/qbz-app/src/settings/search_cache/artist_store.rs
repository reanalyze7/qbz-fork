use super::ARTIST_CACHE_FILE;
use qbz_models::{Album, Artist, Playlist, Track};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// The non-persisted, new-release-sensitive portion of a cached result.
#[derive(Debug, Clone, Default)]
pub(super) struct VolatileSlice {
    pub(super) albums: Vec<Album>,
    pub(super) tracks: Vec<Track>,
    pub(super) playlists: Vec<Playlist>,
}

/// JSON-backed store for the artist slice. Models the graceful-degradation
/// discipline of `discover_prefs.rs` (never panics; a missing/corrupt file
/// yields an empty map) but is a plain `serde_json` read/write of a
/// `HashMap<normalized_query, Vec<Artist>>` rather than SQLite, since the
/// payload is a single small blob with no query needs.
pub(super) struct ArtistCacheStore {
    path: PathBuf,
    entries: HashMap<String, Vec<Artist>>,
}

impl ArtistCacheStore {
    /// Open the store at `<base_dir>/search_artist_cache.json`, loading any
    /// existing entries. A missing directory is created; a missing or corrupt
    /// file degrades to an empty map (never an error to the caller).
    pub(super) fn open_at(base_dir: &Path) -> Self {
        // Best-effort: if the dir can't be created the first save() will retry.
        let _ = std::fs::create_dir_all(base_dir);
        let path = base_dir.join(ARTIST_CACHE_FILE);
        let entries = Self::load_from(&path);
        Self { path, entries }
    }

    fn load_from(path: &Path) -> HashMap<String, Vec<Artist>> {
        let Ok(text) = std::fs::read_to_string(path) else {
            return HashMap::new();
        };
        serde_json::from_str::<HashMap<String, Vec<Artist>>>(&text).unwrap_or_default()
    }

    pub(super) fn get(&self, key: &str) -> Option<&Vec<Artist>> {
        self.entries.get(key)
    }

    /// Upsert the artist slice for `key` and persist the whole map. A write
    /// failure is logged but never propagated (the in-memory map stays correct).
    pub(super) fn put(&mut self, key: String, artists: Vec<Artist>) {
        self.entries.insert(key, artists);
        if let Err(e) = self.persist() {
            log::warn!("search_cache: failed to persist artist cache: {}", e);
        }
    }

    fn persist(&self) -> Result<(), String> {
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir)
                .map_err(|e| format!("Failed to create search cache directory: {}", e))?;
        }
        let text = serde_json::to_string(&self.entries)
            .map_err(|e| format!("Failed to serialize artist cache: {}", e))?;
        std::fs::write(&self.path, text)
            .map_err(|e| format!("Failed to write artist cache: {}", e))?;
        Ok(())
    }
}
