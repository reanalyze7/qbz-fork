use qbz_audio::AudioBackendType;

use crate::tui::screens::audio::cascades::{cascade_on_backend_change, cascade_on_toggle};
use crate::tui::screens::audio::fields::AField;

use super::fixtures::base;

// ---- cascades §3.2.3 items 1-3 (toggle) ----

#[test]
fn passthrough_on_forces_lock_output_off() {
    let mut a = base();
    a.skip_sink_switch = true;
    a.dac_passthrough = true;
    cascade_on_toggle(&mut a, AField::Passthrough);
    assert!(!a.skip_sink_switch, "item 1: passthrough ON forces lock-output off");
}

#[test]
fn passthrough_off_forces_force_bp_off() {
    let mut a = base();
    a.pw_force_bitperfect = true;
    a.dac_passthrough = false;
    cascade_on_toggle(&mut a, AField::Passthrough);
    assert!(!a.pw_force_bitperfect, "item 2: passthrough OFF forces force-BP off");
}

#[test]
fn streaming_only_on_forces_gapless_off() {
    let mut a = base();
    a.gapless_enabled = true;
    a.streaming_only = true;
    cascade_on_toggle(&mut a, AField::StreamingOnly);
    assert!(!a.gapless_enabled, "item 3: streaming-only ON forces gapless off");
}

// ---- cascades §3.2.3 items 4-7 (backend switch) ----

#[test]
fn backend_non_pipewire_forces_passthrough_and_force_bp_off() {
    let mut a = base();
    a.backend = AudioBackendType::PipeWire;
    a.dac_passthrough = true;
    a.pw_force_bitperfect = true;
    a.backend = AudioBackendType::Alsa;
    cascade_on_backend_change(&mut a);
    assert!(!a.dac_passthrough, "item 4");
    assert!(!a.pw_force_bitperfect, "item 4");
}

#[test]
fn backend_non_alsa_forces_exclusive_off() {
    let mut a = base();
    a.exclusive_mode = true;
    a.backend = AudioBackendType::PipeWire;
    cascade_on_backend_change(&mut a);
    assert!(!a.exclusive_mode, "item 5: exclusive is ALSA-only");
}

#[test]
fn backend_alsa_forces_gapless_off() {
    let mut a = base();
    a.gapless_enabled = true;
    a.backend = AudioBackendType::Alsa;
    cascade_on_backend_change(&mut a);
    assert!(!a.gapless_enabled, "item 6");
}

#[test]
fn any_backend_change_resets_device_to_system_default() {
    let mut a = base();
    a.output_device = Some("hw:CARD=D30,DEV=0".to_string());
    a.backend = AudioBackendType::PipeWire;
    cascade_on_backend_change(&mut a);
    assert_eq!(a.output_device, None, "item 7: stale device id must never survive");
}
