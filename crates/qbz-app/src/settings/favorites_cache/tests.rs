use super::*;

fn unique_test_dir(name: &str) -> std::path::PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("qbz-app-{name}-{}-{nonce}", std::process::id()))
}

#[test]
fn favorites_cache_track_ids_roundtrip() {
    let dir = unique_test_dir("favcache-roundtrip");
    let store = FavoritesCacheStore::new_at(&dir).unwrap();

    store.add_favorite_track(1).unwrap();
    store.add_favorite_track(2).unwrap();

    let mut ids = store.get_favorite_track_ids().unwrap();
    ids.sort();
    assert_eq!(ids, vec![1, 2]);
    assert!(store.is_track_favorite(1).unwrap());

    store.remove_favorite_track(1).unwrap();
    assert!(!store.is_track_favorite(1).unwrap());
    assert!(store.is_track_favorite(2).unwrap());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn favorites_cache_add_track_is_idempotent() {
    let dir = unique_test_dir("favcache-idempotent");
    let store = FavoritesCacheStore::new_at(&dir).unwrap();

    store.add_favorite_track(7).unwrap();
    store.add_favorite_track(7).unwrap();

    assert_eq!(store.get_favorite_track_ids().unwrap(), vec![7]);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn favorites_cache_sync_replaces_existing_track_set() {
    let dir = unique_test_dir("favcache-sync");
    let store = FavoritesCacheStore::new_at(&dir).unwrap();

    store.add_favorite_track(1).unwrap();
    store.add_favorite_track(2).unwrap();
    store.sync_favorite_tracks(&[3, 4]).unwrap();

    let mut ids = store.get_favorite_track_ids().unwrap();
    ids.sort();
    assert_eq!(ids, vec![3, 4]);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn favorites_cache_other_entities_roundtrip() {
    let dir = unique_test_dir("favcache-entities");
    let store = FavoritesCacheStore::new_at(&dir).unwrap();

    store.add_favorite_album("abc").unwrap();
    store.add_favorite_artist(11).unwrap();
    store.add_favorite_label(22).unwrap();

    assert!(store.is_album_favorite("abc").unwrap());
    assert!(store.is_artist_favorite(11).unwrap());
    assert!(store.is_label_favorite(22).unwrap());

    store.clear_all().unwrap();
    assert!(store.get_favorite_track_ids().unwrap().is_empty());
    assert!(store.get_favorite_album_ids().unwrap().is_empty());
    assert!(store.get_favorite_artist_ids().unwrap().is_empty());
    assert!(store.get_favorite_label_ids().unwrap().is_empty());
    let _ = std::fs::remove_dir_all(dir);
}
