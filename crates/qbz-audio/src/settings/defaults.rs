//! `impl Default for AudioSettings` — the out-of-the-box defaults, kept
//! separate from the struct definition since the field-by-field rationale
//! comments push this well past a shared file's budget on their own.

use super::types::{default_dsd_mode, AudioSettings};
use crate::{AlsaPlugin, AudioBackendType};
use std::collections::HashMap;

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            output_device: None,
            exclusive_mode: false,
            dac_passthrough: false,
            preferred_sample_rate: None,
            // OOTB default is "System" (Some(SystemDefault)): play through the OS
            // default output, shared with other apps like any normal player — no
            // bit-perfect, no `pactl`. See AudioBackendType::default(). "Auto"
            // (None) and explicit backends (PipeWire / ALSA) are honored as-is;
            // this only sets what a fresh install and the Reset action land on.
            //
            // History: this defaulted to Some(PipeWire) on Linux to dodge a rodio
            // DeviceSink-drop-on-resume race on the CPAL path (#375), but that
            // hard-required `pactl` and froze OOTB playback without it (#470). The
            // #375 race is covered by the cpal 0.17.3 / alsa 0.11 stream-drop
            // fixes, and "System" is the app-like default audiophiles override.
            backend_type: Some(AudioBackendType::default()),
            alsa_plugin: Some(AlsaPlugin::Hw), // Default to hw (bit-perfect)
            alsa_hardware_volume: false, // Disabled by default (maximum compatibility)
            stream_first_track: true, // On by default (opt-out)
            stream_buffer_seconds: 2, // 2 seconds initial buffer
            streaming_only: false, // Disabled by default (cache tracks for instant replay)
            limit_quality_to_device: false, // Opt-in. Off since 1.1.9 (#45); wired to the read-only probe in #638 fix 3
            device_max_sample_rate: None, // Set when device is selected
            device_sample_rate_limits: HashMap::new(), // Per-device limits (empty = no limit)
            normalization_enabled: false, // Off by default — preserves bit-perfect pipeline
            normalization_target_lufs: -14.0, // Spotify/YouTube standard
            gapless_enabled: true, // On by default — works for same-format tracks on all backends
            pw_force_bitperfect: false, // Off by default — experimental PipeWire feature
            sync_audio_on_startup: false, // Off by default — opt-in for stale-settings edge case
            quality_fallback_behavior: "ask".to_string(),
            skip_sink_switch: false, // Off by default — only for JACK/DAW routing setups
            allow_quality_fallback: false, // Off by default — fail rather than silently downgrade
            reserve_dac_while_running: false, // Off by default — opt-in DAC reservation (Lifetime B)
            dsd_mode: default_dsd_mode(), // "convert" — safe on every DAC
            crossfade_seconds: 0.0, // Off by default — preserves strict gapless
        }
    }
}
