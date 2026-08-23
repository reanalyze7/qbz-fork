mod quick_and_clear;
mod round_trip;

use super::*;

pub(super) fn unique_test_dir(name: &str) -> std::path::PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("qbz-app-{name}-{}-{nonce}", std::process::id()))
}

pub(super) fn sample_track() -> PersistedQueueTrack {
    PersistedQueueTrack {
        id: 42,
        title: "Track".to_string(),
        artist: "Artist".to_string(),
        album: "Album".to_string(),
        duration_secs: 300,
        artwork_url: Some("https://example.test/art.jpg".to_string()),
        hires: true,
        bit_depth: Some(24),
        sample_rate: Some(96_000.0),
        is_local: true,
        album_id: Some("album-1".to_string()),
        artist_id: Some(7),
        streamable: false,
        source: Some("mixtape".to_string()),
        parental_warning: true,
        source_item_id_hint: Some("item-1".to_string()),
    }
}

#[test]
fn default_session_values_are_stable() {
    let session = PersistedSessionSnapshot::default();

    assert!(session.playback.queue_tracks.is_empty());
    assert_eq!(session.playback.current_index, None);
    assert_eq!(session.playback.current_position_secs, 0);
    assert_eq!(session.playback.volume, 0.75);
    assert!(!session.playback.shuffle_enabled);
    assert_eq!(session.playback.repeat_mode, "off");
    assert!(!session.playback.was_playing);
    assert_eq!(session.shell_view.last_view, "home");
    assert_eq!(session.shell_view.view_context_id, None);
    assert_eq!(session.shell_view.view_context_type, None);
}

#[test]
fn session_store_uses_wal_and_full_synchronous() {
    let dir = unique_test_dir("session-pragmas");
    let store = SessionStore::new_at(&dir).expect("open store");

    assert_eq!(store.pragma_journal_mode().expect("journal mode"), "wal");
    assert_eq!(store.pragma_synchronous().expect("synchronous"), 2);

    let _ = std::fs::remove_dir_all(dir);
}
