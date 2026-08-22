//! Credential and enabled-flag persistence for `ListenBrainzCache`.

use rusqlite::Result as SqlResult;

use super::ListenBrainzCache;

impl ListenBrainzCache {
    /// Save credentials
    pub fn save_credentials(&self, token: &str, user_name: &str) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO credentials (id, token, user_name) VALUES (1, ?, ?)",
                [token, user_name],
            )
            .map_err(|e| format!("Failed to save credentials: {}", e))?;
        Ok(())
    }

    /// Get saved credentials
    pub fn get_credentials(&self) -> Result<(Option<String>, Option<String>), String> {
        let result: SqlResult<(Option<String>, Option<String>)> = self.conn.query_row(
            "SELECT token, user_name FROM credentials WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        );

        match result {
            Ok((token, user_name)) => Ok((token, user_name)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok((None, None)),
            Err(e) => Err(format!("Failed to get credentials: {}", e)),
        }
    }

    /// Clear credentials
    pub fn clear_credentials(&self) -> Result<(), String> {
        self.conn
            .execute("DELETE FROM credentials", [])
            .map_err(|e| format!("Failed to clear credentials: {}", e))?;
        Ok(())
    }

    /// Check if enabled
    pub fn is_enabled(&self) -> Result<bool, String> {
        let result: SqlResult<String> = self.conn.query_row(
            "SELECT value FROM settings WHERE key = 'enabled'",
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
                "INSERT OR REPLACE INTO settings (key, value) VALUES ('enabled', ?)",
                [value],
            )
            .map_err(|e| format!("Failed to set enabled: {}", e))?;
        Ok(())
    }
}
