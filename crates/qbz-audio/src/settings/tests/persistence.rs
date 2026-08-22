use super::common::{fresh_store, unique_test_dir};
use crate::settings::AudioSettingsStore;

#[test]
fn stream_buffer_seconds_clamps_to_valid_range() {
    let (dir, store) = fresh_store("stream-buffer-clamp");

    store
        .set_stream_buffer_seconds(0)
        .expect("set low buffer");
    assert_eq!(
        store.get_settings().expect("get settings").stream_buffer_seconds,
        1
    );

    store
        .set_stream_buffer_seconds(99)
        .expect("set high buffer");
    assert_eq!(
        store.get_settings().expect("get settings").stream_buffer_seconds,
        10
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn reset_all_preserves_quality_fallback_behavior() {
    let (dir, store) = fresh_store("reset-preserves-quality");
    store
        .set_quality_fallback_behavior("always_skip")
        .expect("set quality fallback");
    store.set_output_device(Some("hw:4,0")).expect("set device");
    store.set_dac_passthrough(true).expect("set dac");

    let reset = store.reset_all().expect("reset settings");
    let settings = store.get_settings().expect("get settings");

    assert_eq!(reset.quality_fallback_behavior, "always_skip");
    assert_eq!(settings.quality_fallback_behavior, "always_skip");
    assert_eq!(settings.output_device, None);
    assert!(!settings.dac_passthrough);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn settings_persist_and_reopen_all_new_fields() {
    let dir = unique_test_dir("persist-new-fields");
    {
        let store = AudioSettingsStore::new_at(&dir).expect("open store");
        store
            .set_sync_audio_on_startup(true)
            .expect("set sync flag");
        store
            .set_quality_fallback_behavior("always_fallback")
            .expect("set quality fallback");
        store
            .set_reserve_dac_while_running(true)
            .expect("set reserve flag");
        store
            .set_allow_quality_fallback(true)
            .expect("set allow fallback");
        store
            .set_skip_sink_switch(true)
            .expect("set skip sink switch");
    }

    let reopened = AudioSettingsStore::new_at(&dir).expect("reopen store");
    let settings = reopened.get_settings().expect("get settings");

    assert!(settings.sync_audio_on_startup);
    assert_eq!(settings.quality_fallback_behavior, "always_fallback");
    assert!(settings.reserve_dac_while_running);
    assert!(settings.allow_quality_fallback);
    assert!(settings.skip_sink_switch);
    let _ = std::fs::remove_dir_all(dir);
}
