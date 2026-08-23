use super::types::OfflineModeSettings;
use super::OfflineModeStore;
use rusqlite::params;

impl OfflineModeStore {
    pub fn get_settings(&self) -> Result<OfflineModeSettings, String> {
        self.conn
            .query_row(
                "SELECT manual_offline_mode,
                        COALESCE(show_network_folders_in_manual_offline, 0)
                 FROM offline_settings WHERE id = 1",
                [],
                |row| {
                    Ok(OfflineModeSettings {
                        manual_offline_mode: row.get::<_, i64>(0)? != 0,
                        show_network_folders_in_manual_offline: row.get::<_, i64>(1)? != 0,
                    })
                },
            )
            .map_err(|e| format!("Failed to get offline settings: {}", e))
    }

    pub fn set_manual_offline_mode(&self, enabled: bool) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE offline_settings SET manual_offline_mode = ?1 WHERE id = 1",
                params![enabled as i64],
            )
            .map_err(|e| format!("Failed to set manual offline mode: {}", e))?;
        Ok(())
    }

    pub fn set_show_network_folders_in_manual_offline(&self, enabled: bool) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE offline_settings SET show_network_folders_in_manual_offline = ?1 WHERE id = 1",
                params![enabled as i64],
            )
            .map_err(|e| format!("Failed to set show network folders in manual offline: {}", e))?;
        Ok(())
    }

    /// Issue #279 snapshot: the user's `stream_first_track` preference stashed
    /// when entering induced offline. `None` = no snapshot active.
    pub fn get_pre_offline_stream_first_track(&self) -> Result<Option<bool>, String> {
        self.conn
            .query_row(
                "SELECT pre_offline_stream_first_track FROM offline_settings WHERE id = 1",
                [],
                |row| row.get::<_, Option<i64>>(0),
            )
            .map(|opt| opt.map(|v| v != 0))
            .map_err(|e| format!("Failed to read pre_offline_stream_first_track: {}", e))
    }

    /// Store (`Some`) on entering induced offline, clear (`None`) on exit.
    pub fn set_pre_offline_stream_first_track(&self, value: Option<bool>) -> Result<(), String> {
        let param: Option<i64> = value.map(|v| v as i64);
        self.conn
            .execute(
                "UPDATE offline_settings SET pre_offline_stream_first_track = ?1 WHERE id = 1",
                params![param],
            )
            .map_err(|e| format!("Failed to set pre_offline_stream_first_track: {}", e))?;
        Ok(())
    }
}
