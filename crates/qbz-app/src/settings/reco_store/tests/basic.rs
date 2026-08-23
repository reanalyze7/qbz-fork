use super::{insert_at, unique_test_dir};
use crate::settings::reco_store::{now_ts, RecoStore};

#[test]
fn schema_creation_is_idempotent() {
    let dir = unique_test_dir("reco-idempotent");
    {
        let store = RecoStore::new_at(&dir).expect("open");
        store.log_play_event(1, Some("a1".into()), Some(10), Some(5)).unwrap();
    }
    // Reopen the SAME db file — init() must not error on existing tables.
    {
        let store = RecoStore::new_at(&dir).expect("reopen");
        // Data survives and is readable.
        assert_eq!(store.get_recent_track_ids(10).unwrap(), vec![1]);
    }
    // The file lives at <base>/reco/events.db (shared with Tauri).
    assert!(dir.join("reco").join("events.db").exists());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn log_and_read_recent_and_favorite() {
    let dir = unique_test_dir("reco-logread");
    let store = RecoStore::new_at(&dir).expect("open");

    store.log_play_event(100, Some("alb".into()), Some(7), Some(3)).unwrap();
    store.log_play_event(200, Some("alb".into()), Some(7), Some(3)).unwrap();
    store.log_favorite_event(300, Some("alb2".into()), Some(9), Some(4)).unwrap();

    let recent = store.get_recent_track_ids(10).unwrap();
    assert!(recent.contains(&100) && recent.contains(&200));
    assert!(!recent.contains(&300)); // favorite is not a play

    let favs = store.get_favorite_track_ids(10).unwrap();
    assert_eq!(favs, vec![300]);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn windowed_recent_query_respects_window() {
    let dir = unique_test_dir("reco-window");
    let store = RecoStore::new_at(&dir).expect("open");
    let now = now_ts();
    let day = 86_400;
    // track 1 played 2 days ago (inside 7d window), track 2 played 10 days ago (outside).
    insert_at(&store, "play", "track", Some(1), Some("a"), Some(11), Some(2), now - 2 * day);
    insert_at(&store, "play", "track", Some(2), Some("b"), Some(12), Some(2), now - 10 * day);

    let week = store.get_recent_track_ids_since(7 * day, 50).unwrap();
    assert_eq!(week, vec![1]);
    // The non-windowed query still sees both.
    let all = store.get_recent_track_ids(50).unwrap();
    assert!(all.contains(&1) && all.contains(&2));
    let _ = std::fs::remove_dir_all(dir);
}
