use super::prefs::normalize_tray_icon_theme;
use super::store::TraySettingsStore;
use rusqlite::params;

impl TraySettingsStore {
    pub fn set_enable_tray(&self, value: bool) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE tray_settings SET enable_tray = ?1 WHERE id = 1",
                params![if value { 1 } else { 0 }],
            )
            .map_err(|e| format!("Failed to set enable_tray: {}", e))?;
        Ok(())
    }

    pub fn set_minimize_to_tray(&self, value: bool) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE tray_settings SET minimize_to_tray = ?1 WHERE id = 1",
                params![if value { 1 } else { 0 }],
            )
            .map_err(|e| format!("Failed to set minimize_to_tray: {}", e))?;
        Ok(())
    }

    pub fn set_close_to_tray(&self, value: bool) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE tray_settings SET close_to_tray = ?1 WHERE id = 1",
                params![if value { 1 } else { 0 }],
            )
            .map_err(|e| format!("Failed to set close_to_tray: {}", e))?;
        Ok(())
    }

    pub fn set_tray_icon_theme(&self, value: &str) -> Result<(), String> {
        let normalized = normalize_tray_icon_theme(value);
        self.conn
            .execute(
                "UPDATE tray_settings SET tray_icon_theme = ?1 WHERE id = 1",
                params![normalized],
            )
            .map_err(|e| format!("Failed to set tray_icon_theme: {}", e))?;
        Ok(())
    }

    pub fn set_mac_hide_dock(&self, value: bool) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE tray_settings SET mac_hide_dock = ?1 WHERE id = 1",
                params![if value { 1 } else { 0 }],
            )
            .map_err(|e| format!("Failed to set mac_hide_dock: {}", e))?;
        Ok(())
    }
}
