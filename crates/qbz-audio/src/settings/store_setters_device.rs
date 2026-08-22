//! Setters/getters for streaming behavior and per-device sample-rate caps.

use super::store_core::AudioSettingsStore;
use rusqlite::params;
use std::collections::HashMap;

impl AudioSettingsStore {
    pub fn set_stream_first_track(&self, enabled: bool) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE audio_settings SET stream_first_track = ?1 WHERE id = 1",
                params![enabled as i64],
            )
            .map_err(|e| format!("Failed to set stream first track: {}", e))?;
        Ok(())
    }

    pub fn set_stream_buffer_seconds(&self, seconds: u8) -> Result<(), String> {
        // Clamp to valid range 1-10
        let clamped = seconds.clamp(1, 10);
        self.conn
            .execute(
                "UPDATE audio_settings SET stream_buffer_seconds = ?1 WHERE id = 1",
                params![clamped as i64],
            )
            .map_err(|e| format!("Failed to set stream buffer seconds: {}", e))?;
        Ok(())
    }

    pub fn set_streaming_only(&self, enabled: bool) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE audio_settings SET streaming_only = ?1 WHERE id = 1",
                params![enabled as i64],
            )
            .map_err(|e| format!("Failed to set streaming only: {}", e))?;
        Ok(())
    }

    pub fn set_limit_quality_to_device(&self, enabled: bool) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE audio_settings SET limit_quality_to_device = ?1 WHERE id = 1",
                params![enabled as i64],
            )
            .map_err(|e| format!("Failed to set limit quality to device: {}", e))?;
        Ok(())
    }

    pub fn set_device_max_sample_rate(&self, rate: Option<u32>) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE audio_settings SET device_max_sample_rate = ?1 WHERE id = 1",
                params![rate.map(|r| r as i64)],
            )
            .map_err(|e| format!("Failed to set device max sample rate: {}", e))?;
        Ok(())
    }

    /// Set the sample rate limit for a specific device
    /// If rate is None, removes the limit for that device
    pub fn set_device_sample_rate_limit(
        &self,
        device_id: &str,
        rate: Option<u32>,
    ) -> Result<(), String> {
        // Get current limits
        let mut limits = self.get_device_sample_rate_limits()?;

        // Update or remove the limit for this device
        if let Some(r) = rate {
            limits.insert(device_id.to_string(), r);
        } else {
            limits.remove(device_id);
        }

        // Serialize and save
        let json = serde_json::to_string(&limits)
            .map_err(|e| format!("Failed to serialize device sample rate limits: {}", e))?;

        self.conn
            .execute(
                "UPDATE audio_settings SET device_sample_rate_limits = ?1 WHERE id = 1",
                params![json],
            )
            .map_err(|e| format!("Failed to set device sample rate limits: {}", e))?;
        Ok(())
    }

    /// Get the sample rate limit for a specific device
    /// Returns None if no limit is set for this device
    pub fn get_device_sample_rate_limit(&self, device_id: &str) -> Result<Option<u32>, String> {
        let limits = self.get_device_sample_rate_limits()?;
        Ok(limits.get(device_id).copied())
    }

    /// Get all device sample rate limits
    fn get_device_sample_rate_limits(&self) -> Result<HashMap<String, u32>, String> {
        let json: Option<String> = self
            .conn
            .query_row(
                "SELECT device_sample_rate_limits FROM audio_settings WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to get device sample rate limits: {}", e))?;

        match json {
            Some(s) if !s.is_empty() => serde_json::from_str(&s)
                .map_err(|e| format!("Failed to parse device sample rate limits: {}", e)),
            _ => Ok(HashMap::new()),
        }
    }
}
