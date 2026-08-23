use std::path::Path;

use serde_json::Value;

use qbz_audio::settings::AudioSettingsStore;
use qbz_audio::AudioBackendType;

use super::apply_writes::as_bool;

pub(super) fn apply_audio_writes(data_root: &Path, writes: &[(&str, &Value)]) -> Result<(), String> {
    let store = AudioSettingsStore::new_at(data_root)?;
    for (key, value) in writes {
        match *key {
            "backend_type" => {
                let b: Option<AudioBackendType> = serde_json::from_value((*value).clone())
                    .map_err(|e| format!("backend_type: {e}"))?;
                store.set_backend_type(b)?;
            }
            "output_device" => {
                let d = value.as_str();
                store.set_output_device(d)?;
            }
            "alsa_plugin" => {
                let p = serde_json::from_value((*value).clone())
                    .map_err(|e| format!("alsa_plugin: {e}"))?;
                store.set_alsa_plugin(p)?;
            }
            "alsa_hardware_volume" => store.set_alsa_hardware_volume(as_bool(value))?,
            "exclusive_mode" => store.set_exclusive_mode(as_bool(value))?,
            "dac_passthrough" => store.set_dac_passthrough(as_bool(value))?,
            "pw_force_bitperfect" => store.set_pw_force_bitperfect(as_bool(value))?,
            "skip_sink_switch" => store.set_skip_sink_switch(as_bool(value))?,
            "reserve_dac_while_running" => store.set_reserve_dac_while_running(as_bool(value))?,
            "dsd_mode" => store.set_dsd_mode(value.as_str().unwrap_or("convert"))?,
            "stream_first_track" => store.set_stream_first_track(as_bool(value))?,
            "stream_buffer_seconds" => {
                store.set_stream_buffer_seconds(value.as_u64().unwrap_or(2) as u8)?
            }
            "streaming_only" => store.set_streaming_only(as_bool(value))?,
            "limit_quality_to_device" => store.set_limit_quality_to_device(as_bool(value))?,
            "preferred_sample_rate" => {
                store.set_sample_rate(value.as_u64().map(|r| r as u32))?
            }
            "normalization_enabled" => store.set_normalization_enabled(as_bool(value))?,
            "normalization_target_lufs" => {
                store.set_normalization_target_lufs(value.as_f64().unwrap_or(-14.0) as f32)?
            }
            "gapless_enabled" => store.set_gapless_enabled(as_bool(value))?,
            "allow_quality_fallback" => store.set_allow_quality_fallback(as_bool(value))?,
            "sync_audio_on_startup" => store.set_sync_audio_on_startup(as_bool(value))?,
            "quality_fallback_behavior" => {
                store.set_quality_fallback_behavior(value.as_str().unwrap_or("always_fallback"))?
            }
            other => log::warn!("[bundle] apply: unhandled audio key {other}"),
        }
    }
    Ok(())
}
