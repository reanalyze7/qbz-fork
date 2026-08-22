//! `AudioSettingsStore::get_settings()` — the single big SELECT + row-mapper.
//!
//! COUPLING: the SELECT column order and the numeric `row.get(N)` indices
//! below are positionally coupled to the `ALTER TABLE ADD COLUMN` order in
//! `schema.rs`. Do not reorder one without the other.

use super::store_core::AudioSettingsStore;
use super::types::{default_dsd_mode, AudioSettings};
use crate::{AlsaPlugin, AudioBackendType};
use std::collections::HashMap;

impl AudioSettingsStore {
    pub fn get_settings(&self) -> Result<AudioSettings, String> {
        self.conn
            .query_row(
                "SELECT output_device, exclusive_mode, dac_passthrough, preferred_sample_rate, backend_type, alsa_plugin, alsa_hardware_volume, stream_first_track, stream_buffer_seconds, streaming_only, limit_quality_to_device, device_max_sample_rate, normalization_enabled, normalization_target_lufs, gapless_enabled, device_sample_rate_limits, pw_force_bitperfect, sync_audio_on_startup, quality_fallback_behavior, skip_sink_switch, allow_quality_fallback, reserve_dac_while_running, dsd_mode, crossfade_seconds FROM audio_settings WHERE id = 1",
                [],
                |row| {
                    // Parse backend_type from JSON string
                    let backend_type: Option<AudioBackendType> = row
                        .get::<_, Option<String>>(4)?
                        .and_then(|s| serde_json::from_str(&s).ok());

                    // Parse alsa_plugin from JSON string
                    let alsa_plugin: Option<AlsaPlugin> = row
                        .get::<_, Option<String>>(5)?
                        .and_then(|s| serde_json::from_str(&s).ok());

                    // Parse device_sample_rate_limits from JSON string
                    let device_sample_rate_limits: HashMap<String, u32> = row
                        .get::<_, Option<String>>(15)?
                        .and_then(|s| serde_json::from_str(&s).ok())
                        .unwrap_or_default();

                    Ok(AudioSettings {
                        output_device: row.get(0)?,
                        exclusive_mode: row.get::<_, i64>(1)? != 0,
                        dac_passthrough: row.get::<_, i64>(2)? != 0,
                        preferred_sample_rate: row.get(3)?,
                        backend_type,
                        alsa_plugin,
                        alsa_hardware_volume: row.get::<_, Option<i64>>(6)?.unwrap_or(0) != 0,
                        stream_first_track: row.get::<_, Option<i64>>(7)?.unwrap_or(0) != 0,
                        stream_buffer_seconds: row.get::<_, Option<i64>>(8)?.unwrap_or(3) as u8,
                        streaming_only: row.get::<_, Option<i64>>(9)?.unwrap_or(0) != 0,
                        limit_quality_to_device: row.get::<_, Option<i64>>(10)?.unwrap_or(0) != 0,
                        device_max_sample_rate: row.get::<_, Option<i64>>(11)?.map(|r| r as u32),
                        device_sample_rate_limits,
                        normalization_enabled: row.get::<_, Option<i64>>(12)?.unwrap_or(0) != 0,
                        normalization_target_lufs: row.get::<_, Option<f64>>(13)?.unwrap_or(-14.0) as f32,
                        gapless_enabled: row.get::<_, Option<i64>>(14)?.unwrap_or(0) != 0,
                        pw_force_bitperfect: row.get::<_, Option<i64>>(16)?.unwrap_or(0) != 0,
                        sync_audio_on_startup: row.get::<_, Option<i64>>(17)?.unwrap_or(0) != 0,
                        quality_fallback_behavior: row
                            .get::<_, Option<String>>(18)?
                            .unwrap_or_else(|| "ask".to_string()),
                        skip_sink_switch: row.get::<_, Option<i64>>(19)?.unwrap_or(0) != 0,
                        allow_quality_fallback: row.get::<_, Option<i64>>(20)?.unwrap_or(0) != 0,
                        reserve_dac_while_running: row
                            .get::<_, Option<i64>>(21)?
                            .unwrap_or(0)
                            != 0,
                        dsd_mode: row
                            .get::<_, Option<String>>(22)?
                            .unwrap_or_else(default_dsd_mode),
                        crossfade_seconds: row.get::<_, Option<f64>>(23)?.unwrap_or(0.0) as f32,
                    })
                },
            )
            .map_err(|e| format!("Failed to get audio settings: {}", e))
    }
}
