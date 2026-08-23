use crate::daemon::reload::audio_routing_changed;

fn base_audio_settings() -> qbz_audio::settings::AudioSettings {
    qbz_audio::settings::AudioSettings::default()
}

#[test]
fn audio_routing_changed_false_when_nothing_moved() {
    let a = base_audio_settings();
    let b = base_audio_settings();
    assert!(!audio_routing_changed(&a, &b));
}

#[test]
fn audio_routing_changed_true_for_each_reinit_class_field() {
    let base = base_audio_settings();

    let mut backend = base.clone();
    backend.backend_type = Some(qbz_audio::AudioBackendType::Alsa);
    assert!(audio_routing_changed(&base, &backend), "backend_type");

    let mut device = base.clone();
    device.output_device = Some("hw:CARD=D30,DEV=0".into());
    assert!(audio_routing_changed(&base, &device), "output_device");

    let mut plugin = base.clone();
    plugin.alsa_plugin = Some(qbz_audio::AlsaPlugin::PlugHw);
    assert!(audio_routing_changed(&base, &plugin), "alsa_plugin");

    let mut hw_vol = base.clone();
    hw_vol.alsa_hardware_volume = !base.alsa_hardware_volume;
    assert!(audio_routing_changed(&base, &hw_vol), "alsa_hardware_volume");

    let mut excl = base.clone();
    excl.exclusive_mode = !base.exclusive_mode;
    assert!(audio_routing_changed(&base, &excl), "exclusive_mode");

    let mut pass = base.clone();
    pass.dac_passthrough = !base.dac_passthrough;
    assert!(audio_routing_changed(&base, &pass), "dac_passthrough");

    let mut lock_out = base.clone();
    lock_out.skip_sink_switch = !base.skip_sink_switch;
    assert!(audio_routing_changed(&base, &lock_out), "skip_sink_switch");

    let mut dsd = base.clone();
    dsd.dsd_mode = "dop".to_string();
    assert!(audio_routing_changed(&base, &dsd), "dsd_mode");

    let mut rate = base.clone();
    rate.device_max_sample_rate = Some(192_000);
    assert!(audio_routing_changed(&base, &rate), "device_max_sample_rate");
}

#[test]
fn audio_routing_changed_false_for_reload_class_fields_only() {
    // Changing ONLY Reload-class fields must never trip a reinit.
    let base = base_audio_settings();
    let mut reload_only = base.clone();
    reload_only.gapless_enabled = !base.gapless_enabled;
    reload_only.stream_first_track = !base.stream_first_track;
    reload_only.stream_buffer_seconds = 7;
    reload_only.streaming_only = !base.streaming_only;
    reload_only.limit_quality_to_device = !base.limit_quality_to_device;
    reload_only.allow_quality_fallback = !base.allow_quality_fallback;
    reload_only.quality_fallback_behavior = "always_skip".to_string();
    reload_only.normalization_enabled = !base.normalization_enabled;
    reload_only.normalization_target_lufs = -18.0;
    reload_only.pw_force_bitperfect = !base.pw_force_bitperfect;
    reload_only.reserve_dac_while_running = !base.reserve_dac_while_running;
    reload_only.sync_audio_on_startup = !base.sync_audio_on_startup;
    assert!(!audio_routing_changed(&base, &reload_only));
}
