//! Qobuz artist validation cache, keyed by normalized name.

use rusqlite::{params, OptionalExtension};

use super::{MusicBrainzCache, QOBUZ_VALIDATION_TTL_SECS};

impl MusicBrainzCache {
    /// Get cached Qobuz validation result for an artist name
    pub fn get_qobuz_validation(&self, name_normalized: &str) -> Result<Option<String>, String> {
        let min_fetched_at = Self::current_timestamp() - QOBUZ_VALIDATION_TTL_SECS;
        let result: Option<String> = self
            .conn
            .query_row(
                "SELECT data FROM mb_qobuz_validation WHERE name_normalized = ? AND fetched_at > ?",
                params![name_normalized, min_fetched_at],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| format!("Failed to query validation cache: {}", e))?;
        Ok(result)
    }

    /// Cache Qobuz validation result
    pub fn set_qobuz_validation(&self, name_normalized: &str, data: &str) -> Result<(), String> {
        let fetched_at = Self::current_timestamp();
        self.conn
            .execute(
                "INSERT OR REPLACE INTO mb_qobuz_validation (name_normalized, data, fetched_at) VALUES (?, ?, ?)",
                params![name_normalized, data, fetched_at],
            )
            .map_err(|e| format!("Failed to cache validation: {}", e))?;
        Ok(())
    }
}
