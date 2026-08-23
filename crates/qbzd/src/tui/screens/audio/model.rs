use qbz_audio::settings::AudioSettings;
use qbz_audio::{AlsaPlugin, AudioBackendType};

// ============================ staged form ============================

#[derive(Debug, Clone, PartialEq)]
pub struct StagedAudio {
    pub backend: AudioBackendType,
    pub output_device: Option<String>,
    pub alsa_plugin: AlsaPlugin,
    pub alsa_hardware_volume: bool,
    pub dsd_mode: String,
    pub exclusive_mode: bool,
    pub reserve_dac: bool,
    pub dac_passthrough: bool,
    pub pw_force_bitperfect: bool,
    pub skip_sink_switch: bool,
    pub stream_first_track: bool,
    pub stream_buffer_seconds: u8,
    pub streaming_only: bool,
    /// Carried (not shown here) so the §3.2.3 cascades that force gapless off
    /// (backend=ALSA, streaming-only=ON) persist through the Audio save.
    pub gapless_enabled: bool,
}

impl StagedAudio {
    pub fn from_settings(a: &AudioSettings) -> Self {
        Self {
            backend: a.backend_type.unwrap_or_default(),
            output_device: a.output_device.clone(),
            alsa_plugin: a.alsa_plugin.unwrap_or_default(),
            alsa_hardware_volume: a.alsa_hardware_volume,
            dsd_mode: a.dsd_mode.clone(),
            exclusive_mode: a.exclusive_mode,
            reserve_dac: a.reserve_dac_while_running,
            dac_passthrough: a.dac_passthrough,
            pw_force_bitperfect: a.pw_force_bitperfect,
            skip_sink_switch: a.skip_sink_switch,
            stream_first_track: a.stream_first_track,
            stream_buffer_seconds: a.stream_buffer_seconds,
            streaming_only: a.streaming_only,
            gapless_enabled: a.gapless_enabled,
        }
    }
}
