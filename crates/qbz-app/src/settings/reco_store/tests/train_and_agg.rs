use super::{insert_at, unique_test_dir};
use crate::settings::reco_store::{
    now_ts, RecoEventInput, RecoEventType, RecoItemType, RecoStore, TrainParams,
};

#[test]
fn train_favorite_outranks_play() {
    let dir = unique_test_dir("reco-train");
    let mut store = RecoStore::new_at(&dir).expect("open");
    let now = now_ts();
    // track 1: a single play (weight 1.0). track 2: a favorite (weight 3.0).
    insert_at(&store, "play", "track", Some(1), Some("a"), Some(1), Some(2), now);
    insert_at(&store, "favorite", "track", Some(2), Some("b"), Some(2), Some(2), now);

    store.train(TrainParams::default()).unwrap();

    // The "all" track scoring should rank the favorited track first.
    let scored = store.get_scored_track_ids("all", 10).unwrap();
    assert_eq!(scored.first(), Some(&2));
    // The "favorite" bucket only contains the favorited track.
    let fav_scored = store.get_scored_track_ids("favorite", 10).unwrap();
    assert_eq!(fav_scored, vec![2]);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn forgotten_favorites_excludes_recently_played() {
    let dir = unique_test_dir("reco-forgotten");
    let store = RecoStore::new_at(&dir).unwrap();
    // Favorite albums A and B; only A was played (now).
    store.log_favorite_event(1, Some("A".into()), Some(10), None).unwrap();
    store.log_favorite_event(2, Some("B".into()), Some(11), None).unwrap();
    store.log_play_event(1, Some("A".into()), Some(10), None).unwrap();
    let forgotten = store.get_forgotten_favorite_album_ids(10, 30).unwrap();
    assert!(forgotten.contains(&"B".to_string())); // never played -> forgotten
    assert!(!forgotten.contains(&"A".to_string())); // played now -> not forgotten
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn known_artists_includes_played_and_favorited() {
    let dir = unique_test_dir("reco-known-artists");
    let store = RecoStore::new_at(&dir).unwrap();
    // artist 10: 3 plays (> threshold 2). artist 20: a single favorite, no
    // plays. artist 30: 1 play, not favorited -> excluded.
    store.log_play_event(1, Some("a".into()), Some(10), None).unwrap();
    store.log_play_event(2, Some("a".into()), Some(10), None).unwrap();
    store.log_play_event(3, Some("a".into()), Some(10), None).unwrap();
    store
        .insert_event(&RecoEventInput {
            event_type: RecoEventType::Favorite,
            item_type: RecoItemType::Artist,
            track_id: None,
            album_id: None,
            artist_id: Some(20),
            playlist_id: None,
            genre_id: None,
        })
        .unwrap();
    store.log_play_event(4, Some("c".into()), Some(30), None).unwrap();
    let known = store.get_known_artist_ids(2).unwrap();
    assert!(known.contains(&10)); // 3 plays > 2
    assert!(known.contains(&20)); // favorited
    assert!(!known.contains(&30)); // 1 play, not favorited
    let _ = std::fs::remove_dir_all(dir);
}
