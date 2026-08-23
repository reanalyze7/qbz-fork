use super::{sample_track, unique_test_dir};
use crate::session_store::*;

#[test]
fn session_store_round_trips_queue_and_shell_view_state() {
    let dir = unique_test_dir("session-round-trip");
    let store = SessionStore::new_at(&dir).expect("open store");
    let session = PersistedSessionSnapshot {
        playback: PersistedPlaybackSession {
            queue_tracks: vec![sample_track()],
            current_index: Some(0),
            current_position_secs: 123,
            volume: 0.42,
            shuffle_enabled: true,
            repeat_mode: "all".to_string(),
            was_playing: true,
            saved_at: 0,
        },
        shell_view: PersistedShellViewState {
            last_view: "album".to_string(),
            view_context_id: Some("album-1".to_string()),
            view_context_type: Some("album".to_string()),
        },
    };

    store.save_session(&session).expect("save session");
    let loaded = store.load_session().expect("load session");

    assert_eq!(loaded.playback.queue_tracks, vec![sample_track()]);
    assert_eq!(loaded.playback.current_index, Some(0));
    assert_eq!(loaded.playback.current_position_secs, 123);
    assert_eq!(loaded.playback.volume, 0.42);
    assert!(loaded.playback.shuffle_enabled);
    assert_eq!(loaded.playback.repeat_mode, "all");
    assert!(loaded.playback.was_playing);
    assert!(loaded.playback.saved_at > 0);
    assert_eq!(loaded.shell_view.last_view, "album");
    assert_eq!(loaded.shell_view.view_context_id.as_deref(), Some("album-1"));
    assert_eq!(loaded.shell_view.view_context_type.as_deref(), Some("album"));

    let _ = std::fs::remove_dir_all(dir);
}
