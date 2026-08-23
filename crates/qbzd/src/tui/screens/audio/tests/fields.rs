use qbz_audio::{AlsaPlugin, AudioBackendType};

use crate::tui::screens::audio::fields::{row_state, AField};
use crate::tui::strings as s;

use super::fixtures::base;

// ---- constraint matrix §3.2.3 ----

#[test]
fn exclusive_enabled_only_on_alsa() {
    let mut a = base();
    a.backend = AudioBackendType::Alsa;
    assert!(row_state(AField::Exclusive, &a).1);
    a.backend = AudioBackendType::PipeWire;
    let (shown, enabled, reason) = row_state(AField::Exclusive, &a);
    assert!(shown && !enabled, "shown-but-disabled off ALSA");
    assert_eq!(reason, Some(s::R_ALSA_ONLY));
}

#[test]
fn passthrough_enabled_only_on_pipewire() {
    let mut a = base();
    a.backend = AudioBackendType::PipeWire;
    assert!(row_state(AField::Passthrough, &a).1);
    a.backend = AudioBackendType::Alsa;
    assert!(!row_state(AField::Passthrough, &a).1);
}

#[test]
fn force_bp_shown_only_when_passthrough_on_and_pipewire() {
    let mut a = base();
    a.backend = AudioBackendType::PipeWire;
    a.dac_passthrough = false;
    assert!(!row_state(AField::ForceBp, &a).0, "hidden when passthrough off");
    a.dac_passthrough = true;
    assert!(row_state(AField::ForceBp, &a).0, "shown when passthrough on + PW");
    a.backend = AudioBackendType::Alsa;
    assert!(!row_state(AField::ForceBp, &a).0, "hidden off PW");
}

#[test]
fn lock_output_shown_on_pipewire_disabled_when_passthrough_on() {
    let mut a = base();
    a.backend = AudioBackendType::PipeWire;
    a.dac_passthrough = false;
    let (shown, enabled, _) = row_state(AField::LockOutput, &a);
    assert!(shown && enabled);
    a.dac_passthrough = true;
    let (shown, enabled, reason) = row_state(AField::LockOutput, &a);
    assert!(shown && !enabled);
    assert_eq!(reason, Some(s::R_PASSTHROUGH_OFF));
}

#[test]
fn alsa_plugin_and_hw_volume_gating() {
    let mut a = base();
    a.backend = AudioBackendType::Alsa;
    a.alsa_plugin = AlsaPlugin::Hw;
    assert!(row_state(AField::AlsaPlugin, &a).0);
    assert!(row_state(AField::HwVolume, &a).0, "hw volume shown on ALSA hw");
    a.alsa_plugin = AlsaPlugin::PlugHw;
    assert!(!row_state(AField::HwVolume, &a).0, "hw volume hidden off hw plugin");
    a.backend = AudioBackendType::PipeWire;
    assert!(!row_state(AField::AlsaPlugin, &a).0, "alsa plugin hidden off ALSA");
}

#[test]
fn buffer_shown_only_when_stream_uncached_on() {
    let mut a = base();
    a.stream_first_track = true;
    assert!(row_state(AField::Buffer, &a).0);
    a.stream_first_track = false;
    assert!(!row_state(AField::Buffer, &a).0);
}
