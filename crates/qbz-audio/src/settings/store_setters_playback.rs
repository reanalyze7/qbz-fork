//! Setters/getters for normalization, gapless, crossfade, DSD, fallback
//! behavior, and other playback-time preferences.

use super::store_core::AudioSettingsStore;
use rusqlite::params;

impl AudioSettingsStore {
    pub fn set_normalization_enabled(&self, enabled: bool) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE audio_settings SET normalization_enabled = ?1 WHERE id = 1",
                params![enabled as i64],
            )
            .map_err(|e| format!("Failed to set normalization enabled: {}", e))?;
        Ok(())
    }

    pub fn set_crossfade_seconds(&self, seconds: f32) -> Result<(), String> {
        let clamped = seconds.clamp(0.0, 10.0) as f64;
        self.conn
            .execute(
                "UPDATE audio_settings SET crossfade_seconds = ?1 WHERE id = 1",
                params![clamped],
            )
            .map_err(|e| format!("Failed to set crossfade seconds: {}", e))?;
        Ok(())
    }

    pub fn set_gapless_enabled(&self, enabled: bool) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE audio_settings SET gapless_enabled = ?1 WHERE id = 1",
                params![enabled as i64],
            )
            .map_err(|e| format!("Failed to set gapless enabled: {}", e))?;
        Ok(())
    }

    pub fn set_allow_quality_fallback(&self, enabled: bool) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE audio_settings SET allow_quality_fallback = ?1 WHERE id = 1",
                params![enabled as i64],
            )
            .map_err(|e| format!("Failed to set allow_quality_fallback: {}", e))?;
        Ok(())
    }

    pub fn set_skip_sink_switch(&self, enabled: bool) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE audio_settings SET skip_sink_switch = ?1 WHERE id = 1",
                params![enabled as i64],
            )
            .map_err(|e| format!("Failed to set skip_sink_switch: {}", e))?;
        Ok(())
    }

    /// Persist the `reserve_dac_while_running` flag (Lifetime B from the
    /// ALSA exclusive-hardening design spec). Toggling this only updates
    /// the DB row; applying the change to the live `DeviceReservation`
    /// guard is the caller's responsibility.
    pub fn set_reserve_dac_while_running(&self, enabled: bool) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE audio_settings SET reserve_dac_while_running = ?1 WHERE id = 1",
                params![enabled as i64],
            )
            .map_err(|e| format!("Failed to set reserve_dac_while_running: {}", e))?;
        Ok(())
    }

    /// Persist the DSD delivery mode ("convert" | "dop" | "native", DSD plan
    /// Phases 2-3). Deliberately NOT part of reset_all's UPDATE.
    pub fn set_dsd_mode(&self, mode: &str) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE audio_settings SET dsd_mode = ?1 WHERE id = 1",
                params![mode],
            )
            .map_err(|e| format!("Failed to set dsd_mode: {}", e))?;
        Ok(())
    }

    pub fn set_pw_force_bitperfect(&self, enabled: bool) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE audio_settings SET pw_force_bitperfect = ?1 WHERE id = 1",
                params![enabled as i64],
            )
            .map_err(|e| format!("Failed to set pw_force_bitperfect: {}", e))?;
        Ok(())
    }

    pub fn set_sync_audio_on_startup(&self, enabled: bool) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE audio_settings SET sync_audio_on_startup = ?1 WHERE id = 1",
                params![enabled as i64],
            )
            .map_err(|e| format!("Failed to set sync_audio_on_startup: {}", e))?;
        Ok(())
    }
}
