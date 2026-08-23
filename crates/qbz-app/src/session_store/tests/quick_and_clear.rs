use super::{sample_track, unique_test_dir};
use crate::session_store::*;

#[test]
fn quick_saves_update_only_targeted_playback_fields() {
    let dir = unique_test_dir("session-quick-save");
    let store = SessionStore::new_at(&dir).expect("open store");

    store.save_position(77).expect("save position");
    store.save_volume(0.25).expect("save volume");
    store
        .save_playback_mode(true, "one")
        .expect("save playback mode");

    let loaded = store.load_session().expect("load session");

    assert_eq!(loaded.playback.current_position_secs, 77);
    assert_eq!(loaded.playback.volume, 0.25);
    assert!(loaded.playback.shuffle_enabled);
    assert_eq!(loaded.playback.repeat_mode, "one");
    assert_eq!(loaded.shell_view.last_view, "home");

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn clear_session_resets_playback_and_shell_view_fields() {
    let dir = unique_test_dir("session-clear");
    let store = SessionStore::new_at(&dir).expect("open store");
    let session = PersistedSessionSnapshot {
        playback: PersistedPlaybackSession {
            queue_tracks: vec![sample_track()],
            current_index: Some(0),
            current_position_secs: 55,
            volume: 0.9,
            shuffle_enabled: true,
            repeat_mode: "all".to_string(),
            was_playing: true,
            saved_at: 0,
        },
        shell_view: PersistedShellViewState {
            last_view: "artist".to_string(),
            view_context_id: Some("7".to_string()),
            view_context_type: Some("artist".to_string()),
        },
    };

    store.save_session(&session).expect("save session");
    store.clear_session().expect("clear session");
    let loaded = store.load_session().expect("load session");

    assert!(loaded.playback.queue_tracks.is_empty());
    assert_eq!(loaded.playback.current_index, None);
    assert_eq!(loaded.playback.current_position_secs, 0);
    assert_eq!(loaded.playback.volume, 0.9);
    assert!(loaded.playback.shuffle_enabled);
    assert_eq!(loaded.playback.repeat_mode, "all");
    assert!(!loaded.playback.was_playing);
    assert_eq!(loaded.shell_view.last_view, "home");
    assert_eq!(loaded.shell_view.view_context_id, None);
    assert_eq!(loaded.shell_view.view_context_type, None);

    let _ = std::fs::remove_dir_all(dir);
}
