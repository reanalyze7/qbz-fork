//! Getter/setter for the ADR-003 quality-fallback-behavior preference, plus
//! the normalization target LUFS setter.

use super::store_core::AudioSettingsStore;
use rusqlite::params;

impl AudioSettingsStore {
    pub fn get_quality_fallback_behavior(&self) -> Result<String, String> {
        let settings = self.get_settings()?;
        let value = &settings.quality_fallback_behavior;
        match value.as_str() {
            "ask" | "always_fallback" | "always_skip" => Ok(value.clone()),
            _ => Ok("ask".to_string()),
        }
    }

    pub fn set_quality_fallback_behavior(&self, behavior: &str) -> Result<(), String> {
        match behavior {
            "ask" | "always_fallback" | "always_skip" => {}
            _ => return Err(format!("Invalid quality_fallback_behavior: {}", behavior)),
        }
        self.conn
            .execute(
                "UPDATE audio_settings SET quality_fallback_behavior = ?1 WHERE id = 1",
                params![behavior],
            )
            .map_err(|e| format!("Failed to set quality_fallback_behavior: {}", e))?;
        Ok(())
    }

    pub fn set_normalization_target_lufs(&self, target: f32) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE audio_settings SET normalization_target_lufs = ?1 WHERE id = 1",
                params![target as f64],
            )
            .map_err(|e| format!("Failed to set normalization target LUFS: {}", e))?;
        Ok(())
    }
}
