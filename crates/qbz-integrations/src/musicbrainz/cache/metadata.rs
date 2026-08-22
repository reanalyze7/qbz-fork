//! Artist metadata cache (location, genres, life span), keyed by MBID.

use rusqlite::{params, OptionalExtension};

use super::super::models::ArtistMetadata;
use super::{MusicBrainzCache, METADATA_TTL_SECS};

impl MusicBrainzCache {
    /// Get cached artist metadata by MBID
    pub fn get_artist_metadata(&self, mbid: &str) -> Result<Option<ArtistMetadata>, String> {
        let min_fetched_at = Self::current_timestamp() - METADATA_TTL_SECS;
        let result: Option<String> = self
            .conn
            .query_row(
                "SELECT data FROM mb_artist_metadata WHERE mbid = ? AND fetched_at > ?",
                params![mbid, min_fetched_at],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| format!("Failed to query metadata cache: {}", e))?;

        if let Some(data) = result {
            serde_json::from_str(&data)
                .map(Some)
                .map_err(|e| format!("Failed to parse cached metadata: {}", e))
        } else {
            Ok(None)
        }
    }

    /// Cache artist metadata
    pub fn set_artist_metadata(&self, mbid: &str, data: &ArtistMetadata) -> Result<(), String> {
        let fetched_at = Self::current_timestamp();
        let json = serde_json::to_string(data)
            .map_err(|e| format!("Failed to serialize metadata: {}", e))?;
        self.conn
            .execute(
                "INSERT OR REPLACE INTO mb_artist_metadata (mbid, data, fetched_at) VALUES (?, ?, ?)",
                params![mbid, json, fetched_at],
            )
            .map_err(|e| format!("Failed to cache metadata: {}", e))?;
        Ok(())
    }
}
