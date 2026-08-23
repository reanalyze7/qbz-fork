use super::labels::{alsa_plugin_value, backend_value};
use super::state::AudioState;

impl AudioState {
    /// Changed dotted `audio.*` keys for the save path (write_one values).
    pub fn save_keys(&self) -> Vec<(String, String)> {
        let b = &self.baseline;
        let a = &self.staged;
        let mut out = Vec::new();
        let mut push = |k: &str, v: String| out.push((format!("audio.{k}"), v));
        if a.backend != b.backend {
            push("backend", backend_value(a.backend).to_string());
        }
        if a.output_device != b.output_device {
            push(
                "device",
                a.output_device.clone().unwrap_or_else(|| "system".to_string()),
            );
        }
        if a.alsa_plugin != b.alsa_plugin {
            push("alsa_plugin", alsa_plugin_value(a.alsa_plugin).to_string());
        }
        if a.alsa_hardware_volume != b.alsa_hardware_volume {
            push("alsa_hardware_volume", a.alsa_hardware_volume.to_string());
        }
        if a.dsd_mode != b.dsd_mode {
            push("dsd_mode", a.dsd_mode.clone());
        }
        if a.exclusive_mode != b.exclusive_mode {
            push("exclusive_mode", a.exclusive_mode.to_string());
        }
        if a.reserve_dac != b.reserve_dac {
            push("reserve_dac_while_running", a.reserve_dac.to_string());
        }
        if a.dac_passthrough != b.dac_passthrough {
            push("dac_passthrough", a.dac_passthrough.to_string());
        }
        if a.pw_force_bitperfect != b.pw_force_bitperfect {
            push("pw_force_bitperfect", a.pw_force_bitperfect.to_string());
        }
        if a.skip_sink_switch != b.skip_sink_switch {
            push("skip_sink_switch", a.skip_sink_switch.to_string());
        }
        if a.stream_first_track != b.stream_first_track {
            push("stream_first_track", a.stream_first_track.to_string());
        }
        if a.stream_buffer_seconds != b.stream_buffer_seconds {
            push("stream_buffer_seconds", a.stream_buffer_seconds.to_string());
        }
        if a.streaming_only != b.streaming_only {
            push("streaming_only", a.streaming_only.to_string());
        }
        if a.gapless_enabled != b.gapless_enabled {
            push("gapless_enabled", a.gapless_enabled.to_string());
        }
        out
    }
}
