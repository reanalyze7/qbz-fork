//! JSON-serialized release cache, keyed by UPC/barcode.

use rusqlite::{params, OptionalExtension};

use super::{MusicBrainzCache, RELEASE_TTL_SECS};

impl MusicBrainzCache {
    /// Get cached release by barcode
    pub fn get_release<T: serde::de::DeserializeOwned>(
        &self,
        barcode: &str,
    ) -> Result<Option<T>, String> {
        let min_fetched_at = Self::current_timestamp() - RELEASE_TTL_SECS;
        let result: Option<String> = self
            .conn
            .query_row(
                "SELECT data FROM mb_releases WHERE barcode = ? AND fetched_at > ?",
                params![barcode, min_fetched_at],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| format!("Failed to query release cache: {}", e))?;

        if let Some(data) = result {
            serde_json::from_str(&data)
                .map(Some)
                .map_err(|e| format!("Failed to parse cached release: {}", e))
        } else {
            Ok(None)
        }
    }

    /// Cache a release
    pub fn set_release<T: serde::Serialize>(&self, barcode: &str, data: &T) -> Result<(), String> {
        let fetched_at = Self::current_timestamp();
        let json = serde_json::to_string(data)
            .map_err(|e| format!("Failed to serialize release: {}", e))?;
        self.conn
            .execute(
                "INSERT OR REPLACE INTO mb_releases (barcode, data, fetched_at) VALUES (?, ?, ?)",
                params![barcode, json, fetched_at],
            )
            .map_err(|e| format!("Failed to cache release: {}", e))?;
        Ok(())
    }
}
