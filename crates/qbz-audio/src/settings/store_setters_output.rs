//! Setters for output device / backend / ALSA plugin selection.

use super::store_core::AudioSettingsStore;
use crate::{AlsaPlugin, AudioBackendType};
use rusqlite::params;

impl AudioSettingsStore {
    pub fn set_output_device(&self, device: Option<&str>) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE audio_settings SET output_device = ?1 WHERE id = 1",
                params![device],
            )
            .map_err(|e| format!("Failed to set output device: {}", e))?;
        Ok(())
    }

    pub fn set_exclusive_mode(&self, enabled: bool) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE audio_settings SET exclusive_mode = ?1 WHERE id = 1",
                params![enabled as i64],
            )
            .map_err(|e| format!("Failed to set exclusive mode: {}", e))?;
        Ok(())
    }

    pub fn set_dac_passthrough(&self, enabled: bool) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE audio_settings SET dac_passthrough = ?1 WHERE id = 1",
                params![enabled as i64],
            )
            .map_err(|e| format!("Failed to set DAC passthrough: {}", e))?;
        Ok(())
    }

    pub fn set_sample_rate(&self, rate: Option<u32>) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE audio_settings SET preferred_sample_rate = ?1 WHERE id = 1",
                params![rate.map(|r| r as i64)],
            )
            .map_err(|e| format!("Failed to set sample rate: {}", e))?;
        Ok(())
    }

    pub fn set_backend_type(&self, backend: Option<AudioBackendType>) -> Result<(), String> {
        let backend_json = backend
            .map(|b| serde_json::to_string(&b))
            .transpose()
            .map_err(|e| format!("Failed to serialize backend type: {}", e))?;

        self.conn
            .execute(
                "UPDATE audio_settings SET backend_type = ?1 WHERE id = 1",
                params![backend_json],
            )
            .map_err(|e| format!("Failed to set backend type: {}", e))?;

        // When switching to ALSA, ensure alsa_plugin has a value (default to Hw)
        if backend == Some(AudioBackendType::Alsa) {
            let current_plugin: Option<String> = self
                .conn
                .query_row(
                    "SELECT alsa_plugin FROM audio_settings WHERE id = 1",
                    [],
                    |row| row.get(0),
                )
                .map_err(|e| format!("Failed to check alsa_plugin: {}", e))?;

            if current_plugin.is_none() {
                log::info!(
                    "[AudioSettings] ALSA backend selected with no plugin set, defaulting to Hw"
                );
                self.set_alsa_plugin(Some(AlsaPlugin::Hw))?;
            }
        }
        Ok(())
    }

    pub fn set_alsa_plugin(&self, plugin: Option<AlsaPlugin>) -> Result<(), String> {
        let plugin_json = plugin
            .map(|p| serde_json::to_string(&p))
            .transpose()
            .map_err(|e| format!("Failed to serialize ALSA plugin: {}", e))?;

        self.conn
            .execute(
                "UPDATE audio_settings SET alsa_plugin = ?1 WHERE id = 1",
                params![plugin_json],
            )
            .map_err(|e| format!("Failed to set ALSA plugin: {}", e))?;
        Ok(())
    }

    pub fn set_alsa_hardware_volume(&self, enabled: bool) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE audio_settings SET alsa_hardware_volume = ?1 WHERE id = 1",
                params![enabled as i64],
            )
            .map_err(|e| format!("Failed to set ALSA hardware volume: {}", e))?;
        Ok(())
    }
}
