//! Scene discovery cache, keyed by area + seed hash.

use rusqlite::{params, OptionalExtension};

use super::super::models::LocationDiscoveryResponse;
use super::{MusicBrainzCache, SCENE_TTL_SECS};

impl MusicBrainzCache {
    /// Get cached scene discovery results
    pub fn get_scene_cache(
        &self,
        cache_key: &str,
    ) -> Result<Option<LocationDiscoveryResponse>, String> {
        let min_fetched_at = Self::current_timestamp() - SCENE_TTL_SECS;
        let result: Option<String> = self
            .conn
            .query_row(
                "SELECT data FROM mb_scene_cache WHERE cache_key = ? AND fetched_at > ?",
                params![cache_key, min_fetched_at],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| format!("Failed to query scene cache: {}", e))?;

        if let Some(data) = result {
            serde_json::from_str(&data)
                .map(Some)
                .map_err(|e| format!("Failed to parse cached scene: {}", e))
        } else {
            Ok(None)
        }
    }

    /// Cache scene discovery results
    pub fn set_scene_cache(
        &self,
        cache_key: &str,
        data: &LocationDiscoveryResponse,
    ) -> Result<(), String> {
        let fetched_at = Self::current_timestamp();
        let json =
            serde_json::to_string(data).map_err(|e| format!("Failed to serialize scene: {}", e))?;
        self.conn
            .execute(
                "INSERT OR REPLACE INTO mb_scene_cache (cache_key, data, fetched_at) VALUES (?, ?, ?)",
                params![cache_key, json, fetched_at],
            )
            .map_err(|e| format!("Failed to cache scene: {}", e))?;
        Ok(())
    }
}
