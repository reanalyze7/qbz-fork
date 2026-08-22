//! MusicBrainz cache for resolved entities and settings
//!
//! SQLite-based cache with TTL expiration for MusicBrainz lookups.
//! Also persists integration settings (enabled state).

mod artist;
mod maintenance;
mod metadata;
mod qobuz_validation;
mod recording;
mod relations;
mod release;
mod resolved_v2_artist;
mod resolved_v2_track;
mod schema;
mod scene;
mod settings;

use rusqlite::Connection;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// TTL for recording cache (30 days)
pub(super) const RECORDING_TTL_SECS: i64 = 30 * 24 * 60 * 60;
/// TTL for artist cache (7 days)
pub(super) const ARTIST_TTL_SECS: i64 = 7 * 24 * 60 * 60;
/// TTL for release cache (30 days)
pub(super) const RELEASE_TTL_SECS: i64 = 30 * 24 * 60 * 60;
/// TTL for artist relationships cache (7 days)
pub(super) const RELATIONS_TTL_SECS: i64 = 7 * 24 * 60 * 60;
/// TTL for artist metadata cache (30 days)
pub(super) const METADATA_TTL_SECS: i64 = 30 * 24 * 60 * 60;
/// TTL for scene discovery cache (30 days)
pub(super) const SCENE_TTL_SECS: i64 = 30 * 24 * 60 * 60;
/// TTL for Qobuz artist validation cache (30 days)
pub(super) const QOBUZ_VALIDATION_TTL_SECS: i64 = 30 * 24 * 60 * 60;

/// Cache statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct CacheStats {
    pub recordings: u64,
    pub artists: u64,
    pub releases: u64,
    pub relations: u64,
}

/// MusicBrainz cache
pub struct MusicBrainzCache {
    pub(super) conn: Connection,
}

impl MusicBrainzCache {
    /// Create a new cache at the given path
    pub fn new(db_path: &Path) -> Result<Self, String> {
        let conn = Connection::open(db_path)
            .map_err(|e| format!("Failed to open MusicBrainz cache: {}", e))?;

        // Enable WAL mode for concurrent read/write (ADR-002)
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
            .map_err(|e| format!("Failed to enable WAL mode: {}", e))?;

        let cache = Self { conn };
        cache.init_schema()?;

        Ok(cache)
    }

    pub(super) fn current_timestamp() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0)
    }

    /// Normalize artist name for consistent cache keys
    pub(crate) fn normalize_name(name: &str) -> String {
        name.to_lowercase()
            .trim()
            .replace(['\'', '"', '.', ','], "")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }
}
