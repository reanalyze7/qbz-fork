//! JSON-serialized recording cache, keyed by ISRC.

use rusqlite::{params, OptionalExtension};

use super::{MusicBrainzCache, RECORDING_TTL_SECS};

impl MusicBrainzCache {
    /// Get cached recording by ISRC (legacy format)
    pub fn get_recording(&self, isrc: &str) -> Result<Option<serde_json::Value>, String> {
        let min_fetched_at = Self::current_timestamp() - RECORDING_TTL_SECS;
        let result: Option<String> = self
            .conn
            .query_row(
                "SELECT data FROM mb_recordings WHERE isrc = ? AND fetched_at > ?",
                params![isrc, min_fetched_at],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| format!("Failed to query recording cache: {}", e))?;

        if let Some(data) = result {
            serde_json::from_str(&data)
                .map(Some)
                .map_err(|e| format!("Failed to parse cached recording: {}", e))
        } else {
            Ok(None)
        }
    }

    /// Cache a recording (JSON-serialized)
    pub fn set_recording<T: serde::Serialize>(&self, isrc: &str, data: &T) -> Result<(), String> {
        let fetched_at = Self::current_timestamp();
        let json = serde_json::to_string(data)
            .map_err(|e| format!("Failed to serialize recording: {}", e))?;
        self.conn
            .execute(
                "INSERT OR REPLACE INTO mb_recordings (isrc, data, fetched_at) VALUES (?, ?, ?)",
                params![isrc, json, fetched_at],
            )
            .map_err(|e| format!("Failed to cache recording: {}", e))?;
        Ok(())
    }
}
