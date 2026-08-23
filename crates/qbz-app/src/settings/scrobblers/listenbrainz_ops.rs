use super::store::ScrobblerSettingsStore;
use rusqlite::params;

impl ScrobblerSettingsStore {
    // --- ListenBrainz ---

    pub fn set_listenbrainz_enabled(&self, value: bool) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE scrobbler_settings SET listenbrainz_enabled = ?1 WHERE id = 1",
                params![if value { 1 } else { 0 }],
            )
            .map_err(|e| format!("Failed to set listenbrainz_enabled: {}", e))?;
        Ok(())
    }

    /// Persist the ListenBrainz token + username together (after `set_token`).
    pub fn set_listenbrainz_token(&self, token: &str, username: &str) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE scrobbler_settings
                 SET listenbrainz_token = ?1, listenbrainz_username = ?2 WHERE id = 1",
                params![token.trim(), username.trim()],
            )
            .map_err(|e| format!("Failed to set listenbrainz token: {}", e))?;
        Ok(())
    }

    /// Sign out of ListenBrainz: clear token + username, keep enable flag.
    pub fn disconnect_listenbrainz(&self) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE scrobbler_settings
                 SET listenbrainz_token = '', listenbrainz_username = '' WHERE id = 1",
                [],
            )
            .map_err(|e| format!("Failed to disconnect ListenBrainz: {}", e))?;
        Ok(())
    }
}
