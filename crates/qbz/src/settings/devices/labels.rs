//! Now-playing output-led labeling.

use qbz_audio::backend::{AlsaPlugin, AudioBackendType};

/// Compute the two now-playing output leds from the real audio constraints:
/// led 1 = the active backend, led 2 = the effective output mode for THAT
/// backend (bit-perfect / exclusive / routed / shared). `*_active` is true
/// when a deliberate, non-shared mode is engaged (colours the led).
pub(in crate::settings) fn output_labels(
    audio: &qbz_audio::settings::AudioSettings,
) -> (String, String, bool, bool) {
    let (backend_label, backend_active) = match audio.backend_type {
        Some(AudioBackendType::PipeWire) => ("PIPEWIRE", true),
        Some(AudioBackendType::Alsa) => ("ALSA", true),
        Some(AudioBackendType::Jack) => ("JACK", true),
        Some(AudioBackendType::Pulse) => ("PULS", true),
        Some(AudioBackendType::SystemDefault) => ("SYST", false),
        None => ("AUTO", false),
    };
    let (mode_label, mode_active) = match audio.backend_type {
        Some(AudioBackendType::PipeWire) => {
            if audio.dac_passthrough {
                ("DACPASS", true)
            } else if audio.pw_force_bitperfect {
                ("BITPERF", true)
            } else {
                ("SHARED", false)
            }
        }
        Some(AudioBackendType::Alsa) => match audio.alsa_plugin {
            Some(AlsaPlugin::Hw) => {
                if audio.exclusive_mode {
                    ("EXCL", true)
                } else {
                    ("DIRECT", true)
                }
            }
            _ => ("SHARED", false),
        },
        Some(AudioBackendType::Jack) => {
            if audio.reserve_dac_while_running {
                ("LOCKED", true)
            } else {
                ("ROUTED", false)
            }
        }
        Some(AudioBackendType::Pulse) => ("SHARED", false),
        Some(AudioBackendType::SystemDefault) | None => ("DEFAULT", false),
    };
    (
        backend_label.to_string(),
        mode_label.to_string(),
        backend_active,
        mode_active,
    )
}
