//! Artist relationships cache, keyed by MBID.

use rusqlite::{params, OptionalExtension};

use super::super::models::ArtistRelationships;
use super::{MusicBrainzCache, RELATIONS_TTL_SECS};

impl MusicBrainzCache {
    /// Get cached artist relationships by MBID
    pub fn get_artist_relations(&self, mbid: &str) -> Result<Option<ArtistRelationships>, String> {
        let min_fetched_at = Self::current_timestamp() - RELATIONS_TTL_SECS;
        let result: Option<String> = self
            .conn
            .query_row(
                "SELECT data FROM mb_artist_relations WHERE mbid = ? AND fetched_at > ?",
                params![mbid, min_fetched_at],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| format!("Failed to query relations cache: {}", e))?;

        if let Some(data) = result {
            serde_json::from_str(&data)
                .map(Some)
                .map_err(|e| format!("Failed to parse cached relations: {}", e))
        } else {
            Ok(None)
        }
    }

    /// Cache artist relationships
    pub fn set_artist_relations(
        &self,
        mbid: &str,
        data: &ArtistRelationships,
    ) -> Result<(), String> {
        let fetched_at = Self::current_timestamp();
        let json = serde_json::to_string(data)
            .map_err(|e| format!("Failed to serialize relations: {}", e))?;
        self.conn
            .execute(
                "INSERT OR REPLACE INTO mb_artist_relations (mbid, data, fetched_at) VALUES (?, ?, ?)",
                params![mbid, json, fetched_at],
            )
            .map_err(|e| format!("Failed to cache relations: {}", e))?;
        Ok(())
    }
}
