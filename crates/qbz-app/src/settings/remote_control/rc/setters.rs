use rusqlite::params;

use super::store::{generate_token, RemoteControlSettingsStore};

impl RemoteControlSettingsStore {
    pub fn set_enabled(&self, enabled: bool) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE remote_control_settings SET enabled = ?1 WHERE id = 1",
                params![if enabled { 1 } else { 0 }],
            )
            .map_err(|e| format!("Failed to set remote control enabled: {}", e))?;
        Ok(())
    }

    pub fn set_port(&self, port: u16) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE remote_control_settings SET port = ?1 WHERE id = 1",
                params![port as i64],
            )
            .map_err(|e| format!("Failed to set remote control port: {}", e))?;
        Ok(())
    }

    pub fn set_secure(&self, secure: bool) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE remote_control_settings SET secure = ?1 WHERE id = 1",
                params![if secure { 1 } else { 0 }],
            )
            .map_err(|e| format!("Failed to set remote control secure: {}", e))?;
        Ok(())
    }

    pub fn set_token(&self, token: &str) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE remote_control_settings SET token = ?1 WHERE id = 1",
                params![token],
            )
            .map_err(|e| format!("Failed to set remote control token: {}", e))?;
        Ok(())
    }

    pub fn regenerate_token(&self) -> Result<String, String> {
        let token = generate_token();
        self.set_token(&token)?;
        Ok(token)
    }
}
