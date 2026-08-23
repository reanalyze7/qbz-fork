use super::*;
use rusqlite::Connection;
use std::sync::atomic::{AtomicU64, Ordering};

fn unique_test_dir(name: &str) -> std::path::PathBuf {
    static NONCE: AtomicU64 = AtomicU64::new(0);
    let nonce = NONCE.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("qbz-app-{name}-{}-{nonce}", std::process::id()))
}

#[test]
fn defaults_are_online_and_no_network_folders() {
    let dir = unique_test_dir("offline-store-defaults");
    let store = OfflineModeStore::new_at(&dir).unwrap();

    let settings = store.get_settings().unwrap();
    assert!(!settings.manual_offline_mode);
    assert!(!settings.show_network_folders_in_manual_offline);
    assert_eq!(store.get_pre_offline_stream_first_track().unwrap(), None);

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn manual_flag_round_trips() {
    let dir = unique_test_dir("offline-store-manual");
    let store = OfflineModeStore::new_at(&dir).unwrap();

    store.set_manual_offline_mode(true).unwrap();
    assert!(store.get_settings().unwrap().manual_offline_mode);
    store.set_manual_offline_mode(false).unwrap();
    assert!(!store.get_settings().unwrap().manual_offline_mode);

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn network_folders_flag_round_trips() {
    let dir = unique_test_dir("offline-store-netfolders");
    let store = OfflineModeStore::new_at(&dir).unwrap();

    store.set_show_network_folders_in_manual_offline(true).unwrap();
    assert!(
        store
            .get_settings()
            .unwrap()
            .show_network_folders_in_manual_offline
    );

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn scrobble_queue_round_trips_and_marks_sent() {
    let dir = unique_test_dir("offline-store-scrobbles");
    let store = OfflineModeStore::new_at(&dir).unwrap();

    let id1 = store
        .queue_scrobble("Artist A", "Track 1", Some("Album X"), 1000)
        .unwrap();
    let id2 = store.queue_scrobble("Artist B", "Track 2", None, 2000).unwrap();
    assert_ne!(id1, id2);
    assert_eq!(store.queued_scrobble_count().unwrap(), 2);

    let pending = store.get_queued_scrobbles(50).unwrap();
    assert_eq!(pending.len(), 2);
    // Oldest first.
    assert_eq!(pending[0].timestamp, 1000);
    assert_eq!(pending[0].artist, "Artist A");
    assert_eq!(pending[0].album.as_deref(), Some("Album X"));
    assert_eq!(pending[1].album, None);

    store.mark_scrobbles_sent(&[pending[0].id]).unwrap();
    assert_eq!(store.queued_scrobble_count().unwrap(), 1);
    let remaining = store.get_queued_scrobbles(50).unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].timestamp, 2000);

    // Cleanup only touches SENT rows older than the cutoff — the fresh
    // unsent row always survives (the just-sent row's created_at is "now",
    // so it is not older than any cutoff either).
    let _ = store.cleanup_sent_scrobbles(7).unwrap();
    assert_eq!(store.queued_scrobble_count().unwrap(), 1);

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn stream_first_snapshot_round_trips() {
    let dir = unique_test_dir("offline-store-snapshot");
    let store = OfflineModeStore::new_at(&dir).unwrap();

    store.set_pre_offline_stream_first_track(Some(true)).unwrap();
    assert_eq!(store.get_pre_offline_stream_first_track().unwrap(), Some(true));
    store.set_pre_offline_stream_first_track(None).unwrap();
    assert_eq!(store.get_pre_offline_stream_first_track().unwrap(), None);

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn reopens_tauri_era_database_without_data_loss() {
    // Simulate a DB created by the original Tauri schema (pre-migration
    // base tables only), then reopen with this store: migrations must be
    // additive and the existing flag must survive.
    let dir = unique_test_dir("offline-store-compat");
    std::fs::create_dir_all(&dir).unwrap();
    {
        let conn = Connection::open(dir.join("offline_settings.db")).unwrap();
        conn.execute_batch(
            "CREATE TABLE offline_settings (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                manual_offline_mode INTEGER NOT NULL DEFAULT 0,
                show_partial_playlists INTEGER NOT NULL DEFAULT 1
            );
            INSERT INTO offline_settings (id, manual_offline_mode, show_partial_playlists)
            VALUES (1, 1, 1);",
        )
        .unwrap();
    }

    let store = OfflineModeStore::new_at(&dir).unwrap();
    let settings = store.get_settings().unwrap();
    assert!(settings.manual_offline_mode, "Tauri-era flag must survive");
    assert!(!settings.show_network_folders_in_manual_offline);

    let _ = std::fs::remove_dir_all(dir);
}
