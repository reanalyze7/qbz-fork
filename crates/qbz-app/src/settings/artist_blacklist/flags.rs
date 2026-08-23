use std::sync::atomic::Ordering;

use rusqlite::params;

use super::{BlacklistService, BlacklistSettings};

impl BlacklistService {
    /// Set the enabled state.
    ///
    /// Shared by both axes: also gates `is_album_blacklisted()` in `albums.rs`.
    pub fn set_enabled(&self, enabled: bool) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE blacklist_settings SET enabled = ?1 WHERE id = 1",
                params![if enabled { 1 } else { 0 }],
            )
            .map_err(|e| format!("Failed to update enabled setting: {}", e))?;

        self.enabled.store(enabled, Ordering::Relaxed);
        log::info!("[Blacklist] Feature enabled set to: {}", enabled);
        Ok(())
    }

    /// Check if the feature is enabled.
    #[inline]
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    /// Get current settings.
    pub fn get_settings(&self) -> BlacklistSettings {
        BlacklistSettings {
            enabled: self.is_enabled(),
        }
    }
}
