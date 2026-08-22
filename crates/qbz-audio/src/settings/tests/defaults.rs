use super::common::fresh_store;
use crate::settings::AudioSettings;
use crate::{AlsaPlugin, AudioBackendType};

#[test]
fn audio_settings_default_values_are_stable() {
    let settings = AudioSettings::default();

    // OOTB default is "System" — the OS default output (#470).
    assert_eq!(settings.backend_type, Some(AudioBackendType::SystemDefault));
    assert_eq!(settings.alsa_plugin, Some(AlsaPlugin::Hw));
    assert!(settings.gapless_enabled);
    assert!(!settings.sync_audio_on_startup);
    assert_eq!(settings.quality_fallback_behavior, "ask");
    assert!(!settings.skip_sink_switch);
    assert!(!settings.allow_quality_fallback);
    assert!(!settings.reserve_dac_while_running);
}

#[test]
fn audio_settings_store_returns_current_defaults() {
    let (dir, store) = fresh_store("defaults");

    let settings = store.get_settings().expect("get settings");

    // Fresh store is seeded with the OOTB default backend "System" (#470).
    assert_eq!(settings.backend_type, Some(AudioBackendType::SystemDefault));
    assert_eq!(settings.alsa_plugin, None);
    assert!(!settings.gapless_enabled);
    assert_eq!(settings.quality_fallback_behavior, "ask");
    assert!(!settings.reserve_dac_while_running);
    let _ = std::fs::remove_dir_all(dir);
}
