//! Plain, `Send` settings data built off the UI thread.

pub struct SettingsSnapshot {
    // Audio — dropdowns.
    pub(super) streaming_qualities: Vec<String>,
    pub(super) streaming_quality_index: i32,
    pub(super) backends: Vec<String>,
    pub(super) backend_index: i32,
    pub(super) devices: Vec<String>,
    pub(super) device_bp: Vec<bool>,
    pub(super) device_groups: Vec<String>,
    pub(super) device_index: i32,
    pub(super) alsa_plugins: Vec<String>,
    pub(super) alsa_plugin_index: i32,
    // Audio — toggles.
    pub(super) limit_quality_to_device: bool,
    // Detected local device limit (#638 fix 3): the read-only value line
    // ("192 kHz · Hi-Res+"; empty = none) + whether it came from real
    // detection (false = fallback set → the Settings caveat shows).
    pub(super) device_cap_summary: String,
    pub(super) device_cap_detected: bool,
    pub(super) alsa_hardware_volume: bool,
    pub(super) dsd_modes: Vec<String>,
    pub(super) dsd_mode_index: i32,
    pub(super) exclusive_mode: bool,
    pub(super) reserve_dac: bool,
    pub(super) dac_passthrough: bool,
    pub(super) pw_force_bitperfect: bool,
    pub(super) allow_quality_fallback: bool,
    pub(super) sync_audio_on_startup: bool,
    pub(super) skip_sink_switch: bool,
    // Audio — conditional flags.
    pub(super) backend_is_alsa: bool,
    pub(super) backend_is_pipewire: bool,
    pub(super) backend_is_jack: bool,
    pub(super) alsa_plugin_is_hw: bool,
    // Playback.
    pub(super) continue_playback: bool,
    pub(super) show_context_icon: bool,
    pub(super) persist_session: bool,
    pub(super) resume_position: bool,
    pub(super) gapless: bool,
    pub(super) stream_uncached: bool,
    pub(super) streaming_only: bool,
    pub(super) normalization: bool,
    pub(super) buffer_seconds: i32,
    pub(super) crossfade_seconds: i32,
    pub(super) retry_behaviors: Vec<String>,
    pub(super) retry_behavior_index: i32,
    // Now-playing output indicators (backend + effective bit-perfect mode).
    pub(super) output_backend_label: String,
    pub(super) output_mode_label: String,
    pub(super) output_backend_active: bool,
    pub(super) output_mode_active: bool,
}
