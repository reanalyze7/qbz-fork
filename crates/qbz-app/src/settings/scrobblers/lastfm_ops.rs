use super::store::ScrobblerSettingsStore;
use rusqlite::params;

impl ScrobblerSettingsStore {
    // --- Last.fm ---

    pub fn set_lastfm_enabled(&self, value: bool) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE scrobbler_settings SET lastfm_enabled = ?1 WHERE id = 1",
                params![if value { 1 } else { 0 }],
            )
            .map_err(|e| format!("Failed to set lastfm_enabled: {}", e))?;
        Ok(())
    }

    /// Persist the Last.fm session key + username together (after `get_session`).
    pub fn set_lastfm_session(&self, key: &str, username: &str) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE scrobbler_settings
                 SET lastfm_session_key = ?1, lastfm_username = ?2 WHERE id = 1",
                params![key.trim(), username.trim()],
            )
            .map_err(|e| format!("Failed to set lastfm session: {}", e))?;
        Ok(())
    }

    /// Sign out of Last.fm: clear key + username, keep `lastfm_enabled` flag.
    pub fn disconnect_lastfm(&self) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE scrobbler_settings
                 SET lastfm_session_key = '', lastfm_username = '' WHERE id = 1",
                [],
            )
            .map_err(|e| format!("Failed to disconnect Last.fm: {}", e))?;
        Ok(())
    }
}
