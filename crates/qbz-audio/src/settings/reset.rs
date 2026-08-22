//! `AudioSettingsStore::reset_all()` — restore every audio setting to its
//! default, except `quality_fallback_behavior` (ADR-003), which is
//! deliberately preserved across a reset by saving it before the UPDATE and
//! rewriting it afterwards. Do not "simplify away" that two-step dance.

use super::store_core::AudioSettingsStore;
use super::types::AudioSettings;
use rusqlite::params;

impl AudioSettingsStore {
    /// Reset all audio settings to their default values
    pub fn reset_all(&self) -> Result<AudioSettings, String> {
        // ADR-003: quality_fallback_behavior must survive reset_all()
        let saved_fallback = self
            .get_quality_fallback_behavior()
            .unwrap_or_else(|_| "ask".to_string());

        let defaults = AudioSettings::default();
        let backend_json: Option<String> = defaults
            .backend_type
            .map(|b| serde_json::to_string(&b))
            .transpose()
            .map_err(|e| format!("Failed to serialize backend type: {}", e))?;
        let plugin_json: Option<String> = defaults
            .alsa_plugin
            .map(|p| serde_json::to_string(&p))
            .transpose()
            .map_err(|e| format!("Failed to serialize ALSA plugin: {}", e))?;

        // Serialize per-device limits (empty on reset)
        let limits_json = serde_json::to_string(&defaults.device_sample_rate_limits)
            .map_err(|e| format!("Failed to serialize device sample rate limits: {}", e))?;

        self.conn
            .execute(
                "UPDATE audio_settings SET
                    output_device = ?1,
                    exclusive_mode = ?2,
                    dac_passthrough = ?3,
                    preferred_sample_rate = ?4,
                    backend_type = ?5,
                    alsa_plugin = ?6,
                    alsa_hardware_volume = ?7,
                    stream_first_track = ?8,
                    stream_buffer_seconds = ?9,
                    streaming_only = ?10,
                    limit_quality_to_device = ?11,
                    device_max_sample_rate = ?12,
                    normalization_enabled = ?13,
                    normalization_target_lufs = ?14,
                    gapless_enabled = ?15,
                    device_sample_rate_limits = ?16,
                    pw_force_bitperfect = ?17,
                    sync_audio_on_startup = ?18,
                    skip_sink_switch = ?19,
                    allow_quality_fallback = ?20,
                    reserve_dac_while_running = ?21
                WHERE id = 1",
                params![
                    defaults.output_device,
                    defaults.exclusive_mode as i64,
                    defaults.dac_passthrough as i64,
                    defaults.preferred_sample_rate.map(|r| r as i64),
                    backend_json,
                    plugin_json,
                    defaults.alsa_hardware_volume as i64,
                    defaults.stream_first_track as i64,
                    defaults.stream_buffer_seconds as i64,
                    defaults.streaming_only as i64,
                    defaults.limit_quality_to_device as i64,
                    defaults.device_max_sample_rate.map(|r| r as i64),
                    defaults.normalization_enabled as i64,
                    defaults.normalization_target_lufs as f64,
                    defaults.gapless_enabled as i64,
                    limits_json,
                    defaults.pw_force_bitperfect as i64,
                    defaults.sync_audio_on_startup as i64,
                    defaults.skip_sink_switch as i64,
                    defaults.allow_quality_fallback as i64,
                    defaults.reserve_dac_while_running as i64,
                ],
            )
            .map_err(|e| format!("Failed to reset audio settings: {}", e))?;

        // ADR-003: restore quality_fallback_behavior after reset (it is not an audio config)
        self.conn
            .execute(
                "UPDATE audio_settings SET quality_fallback_behavior = ?1 WHERE id = 1",
                params![saved_fallback],
            )
            .map_err(|e| {
                format!(
                    "Failed to restore quality_fallback_behavior after reset: {}",
                    e
                )
            })?;

        let mut result = defaults;
        result.quality_fallback_behavior = saved_fallback;
        Ok(result)
    }
}
