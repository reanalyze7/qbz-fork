use super::common::fresh_store;
use crate::settings::{AudioSettings, AudioSettingsStore};
use crate::{AlsaPlugin, AudioBackendType};

#[test]
fn backend_null_stays_auto_on_reopen() {
    // A NULL backend_type means "Auto" (system default output). It must be
    // preserved across restarts — never backfilled to a concrete backend on
    // store open (the old #375 backfill hard-coded PipeWire and froze OOTB
    // playback on hosts without `pactl`, #470). Only the explicit Reset
    // action rewrites settings.
    let dir = super::common::unique_test_dir("backend-null-auto");
    {
        let store = AudioSettingsStore::new_at(&dir).expect("open store");
        store
            .conn
            .execute(
                "UPDATE audio_settings SET backend_type = NULL WHERE id = 1",
                [],
            )
            .expect("force null (Auto) backend");
    }

    let reopened = AudioSettingsStore::new_at(&dir).expect("reopen store");
    let settings = reopened.get_settings().expect("get settings");

    assert_eq!(settings.backend_type, None);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn switching_to_alsa_sets_default_plugin_when_missing() {
    let (dir, store) = fresh_store("alsa-plugin-default");
    store.set_alsa_plugin(None).expect("clear alsa plugin");

    store
        .set_backend_type(Some(AudioBackendType::Alsa))
        .expect("set backend");
    let settings = store.get_settings().expect("get settings");

    assert_eq!(settings.backend_type, Some(AudioBackendType::Alsa));
    assert_eq!(settings.alsa_plugin, Some(AlsaPlugin::Hw));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn quality_fallback_invalid_value_reads_as_ask() {
    let (dir, store) = fresh_store("quality-invalid");
    store
        .conn
        .execute(
            "UPDATE audio_settings SET quality_fallback_behavior = 'bad-value' WHERE id = 1",
            [],
        )
        .expect("write invalid fallback behavior");

    assert_eq!(
        store
            .get_quality_fallback_behavior()
            .expect("get quality fallback"),
        "ask"
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn deserializes_legacy_json_without_reserve_dac_field() {
    let legacy = r#"{
        "output_device": null,
        "exclusive_mode": false,
        "dac_passthrough": false,
        "preferred_sample_rate": null,
        "backend_type": null,
        "alsa_plugin": null,
        "alsa_hardware_volume": false,
        "stream_first_track": false,
        "stream_buffer_seconds": 3,
        "streaming_only": false,
        "limit_quality_to_device": false,
        "device_max_sample_rate": null,
        "normalization_enabled": false,
        "normalization_target_lufs": -14.0,
        "gapless_enabled": true,
        "pw_force_bitperfect": false,
        "sync_audio_on_startup": false,
        "quality_fallback_behavior": "ask",
        "skip_sink_switch": false,
        "allow_quality_fallback": false
    }"#;

    let settings: AudioSettings =
        serde_json::from_str(legacy).expect("legacy JSON should deserialize");

    assert!(!settings.reserve_dac_while_running);
}
