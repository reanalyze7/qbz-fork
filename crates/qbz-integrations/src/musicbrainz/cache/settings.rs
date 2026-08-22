//! Enabled/disabled persistence for the MusicBrainz integration.

use super::MusicBrainzCache;

impl MusicBrainzCache {
    /// Check if MusicBrainz is enabled
    pub fn is_enabled(&self) -> Result<bool, String> {
        let result: rusqlite::Result<String> = self.conn.query_row(
            "SELECT value FROM mb_settings WHERE key = 'enabled'",
            [],
            |row| row.get(0),
        );
        match result {
            Ok(val) => Ok(val != "0"),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(true), // Default enabled
            Err(e) => Err(format!("Failed to get enabled state: {}", e)),
        }
    }

    /// Set enabled state
    pub fn set_enabled(&self, enabled: bool) -> Result<(), String> {
        let value = if enabled { "1" } else { "0" };
        self.conn
            .execute(
                "INSERT OR REPLACE INTO mb_settings (key, value) VALUES ('enabled', ?)",
                [value],
            )
            .map_err(|e| format!("Failed to set enabled: {}", e))?;
        Ok(())
    }
}
