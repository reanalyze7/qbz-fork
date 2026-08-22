//! Legacy JSON-serialized artist cache, keyed by normalized name.
//!
//! Distinct from the V2 structured `get_artist`/`put_artist` in
//! `resolved_v2_artist.rs`, which uses `.to_lowercase()` directly instead of
//! [`MusicBrainzCache::normalize_name`].

use rusqlite::{params, OptionalExtension};

use super::{MusicBrainzCache, ARTIST_TTL_SECS};

impl MusicBrainzCache {
    /// Get cached artist by name (JSON-serialized)
    pub fn get_artist_by_name<T: serde::de::DeserializeOwned>(
        &self,
        name: &str,
    ) -> Result<Option<T>, String> {
        let normalized = Self::normalize_name(name);
        let min_fetched_at = Self::current_timestamp() - ARTIST_TTL_SECS;
        let result: Option<String> = self
            .conn
            .query_row(
                "SELECT data FROM mb_artists WHERE name_normalized = ? AND fetched_at > ?",
                params![normalized, min_fetched_at],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| format!("Failed to query artist cache: {}", e))?;

        if let Some(data) = result {
            serde_json::from_str(&data)
                .map(Some)
                .map_err(|e| format!("Failed to parse cached artist: {}", e))
        } else {
            Ok(None)
        }
    }

    /// Cache an artist (JSON-serialized)
    pub fn set_artist_by_name<T: serde::Serialize>(
        &self,
        name: &str,
        data: &T,
    ) -> Result<(), String> {
        let normalized = Self::normalize_name(name);
        let fetched_at = Self::current_timestamp();
        let json = serde_json::to_string(data)
            .map_err(|e| format!("Failed to serialize artist: {}", e))?;
        self.conn
            .execute(
                "INSERT OR REPLACE INTO mb_artists (name_normalized, data, fetched_at) VALUES (?, ?, ?)",
                params![normalized, json, fetched_at],
            )
            .map_err(|e| format!("Failed to cache artist: {}", e))?;
        Ok(())
    }
}
