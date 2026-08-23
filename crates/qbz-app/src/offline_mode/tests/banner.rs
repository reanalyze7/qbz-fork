use super::*;
use qbz_audio::settings::AudioSettingsStore;

#[test]
fn offline_session_is_real_offline_even_with_connectivity_up() {
    let _gate = serialize();
    let engine = OfflineModeEngine::new();
    engine.on_connectivity(up());
    engine.set_offline_session(true);

    let status = engine.status();
    assert_eq!(status.mode, OfflineMode::RealOffline);
    assert!(status.show_recovery_banner(), "banner: session offline but net is back");

    engine.set_offline_session(false);
    assert_eq!(engine.status().mode, OfflineMode::Online);
}

#[test]
fn no_banner_while_connectivity_down_or_induced() {
    let _gate = serialize();
    let dir = unique_test_dir("engine-banner");
    let engine = OfflineModeEngine::new();
    engine.init_for_user(&dir).unwrap();

    engine.set_offline_session(true);
    engine.on_connectivity(down());
    assert!(!engine.status().show_recovery_banner());

    engine.on_connectivity(up());
    engine.set_induced(true, None).unwrap();
    assert!(!engine.status().show_recovery_banner());

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn stream_first_snapshot_is_stashed_and_restored() {
    let _gate = serialize();
    let dir = unique_test_dir("engine-279");
    let audio_dir = unique_test_dir("engine-279-audio");
    std::fs::create_dir_all(&audio_dir).unwrap();
    let audio = AudioSettingsStore::new_at(&audio_dir).unwrap();
    audio.set_stream_first_track(true).unwrap();

    let engine = OfflineModeEngine::new();
    engine.init_for_user(&dir).unwrap();

    engine.set_induced(true, Some(&audio)).unwrap();
    assert!(!audio.get_settings().unwrap().stream_first_track, "#279: forced false");

    engine.set_induced(false, Some(&audio)).unwrap();
    assert!(audio.get_settings().unwrap().stream_first_track, "#279: restored");

    let _ = std::fs::remove_dir_all(dir);
    let _ = std::fs::remove_dir_all(audio_dir);
}
