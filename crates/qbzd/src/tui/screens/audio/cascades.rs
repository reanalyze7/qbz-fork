use qbz_audio::AudioBackendType;

use super::fields::AField;
use super::model::StagedAudio;

// ============================ cascades (§3.2.3) ============================

/// Toggle cascades (items 1-3), fired the moment a toggle flips.
pub fn cascade_on_toggle(a: &mut StagedAudio, field: AField) {
    match field {
        AField::Passthrough => {
            if a.dac_passthrough {
                a.skip_sink_switch = false; // item 1: mutually exclusive
            } else {
                a.pw_force_bitperfect = false; // item 2
            }
        }
        AField::StreamingOnly => {
            if a.streaming_only {
                a.gapless_enabled = false; // item 3
            }
        }
        _ => {}
    }
}

/// Backend-switch cascades (items 4-7), fired when Backend changes. The device
/// reset (item 7) means the caller must re-enumerate for the new backend.
pub fn cascade_on_backend_change(a: &mut StagedAudio) {
    if a.backend != AudioBackendType::PipeWire {
        a.dac_passthrough = false; // item 4
        a.pw_force_bitperfect = false;
    }
    if a.backend != AudioBackendType::Alsa {
        a.exclusive_mode = false; // item 5
    }
    if a.backend == AudioBackendType::Alsa {
        a.gapless_enabled = false; // item 6
    }
    a.output_device = None; // item 7: never carry the old backend's device id
}
