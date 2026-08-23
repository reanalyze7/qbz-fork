use super::{insert_at, unique_test_dir};
use crate::settings::reco_store::{now_ts, HomeSeedLimits, RecoStore, TrainParams};

#[test]
fn top_genres_ranking_and_names() {
    let dir = unique_test_dir("reco-genres");
    let store = RecoStore::new_at(&dir).expect("open");
    let now = now_ts();
    // genre 5 played 3x, genre 6 played 1x; genre 0 ignored (> 0 filter).
    insert_at(&store, "play", "track", Some(1), Some("alb5"), Some(1), Some(5), now);
    insert_at(&store, "play", "track", Some(2), Some("alb5"), Some(1), Some(5), now);
    insert_at(&store, "play", "track", Some(3), Some("alb5"), Some(1), Some(5), now);
    insert_at(&store, "play", "track", Some(4), Some("alb6"), Some(2), Some(6), now);
    insert_at(&store, "play", "track", Some(5), Some("albz"), Some(3), Some(0), now);
    // Provide a genre name for the album associated with genre 5.
    store.set_album_genre_name("alb5", "Jazz").unwrap();

    let genres = store.get_top_genres(10).unwrap();
    assert_eq!(genres.len(), 2); // genre 0 excluded
    assert_eq!(genres[0].0, 5); // most played first
    assert_eq!(genres[0].1, "Jazz"); // name resolved from reco_album_meta
    assert_eq!(genres[1].0, 6);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn home_seeds_shape_fallback_and_trained() {
    let dir = unique_test_dir("reco-homeseeds");
    let mut store = RecoStore::new_at(&dir).expect("open");
    let now = now_ts();
    insert_at(&store, "play", "track", Some(1), Some("alb1"), Some(10), Some(2), now - 100);
    insert_at(&store, "play", "track", Some(2), Some("alb2"), Some(11), Some(2), now - 50);
    insert_at(&store, "favorite", "track", Some(9), Some("alb9"), Some(20), Some(3), now - 10);

    // No scores yet -> fallback path.
    let seeds = store.get_home_seeds(HomeSeedLimits::default()).unwrap();
    assert!(seeds.continue_listening_track_ids.contains(&1));
    assert!(seeds.continue_listening_track_ids.contains(&2));
    assert!(seeds.favorite_track_ids.contains(&9));
    assert!(seeds.recently_played_album_ids.contains(&"alb1".to_string()));
    assert!(seeds.top_artist_ids.iter().any(|s| s.artist_id == 10));

    // Train -> scores now exist; seeds still return a coherent shape.
    store.train(TrainParams::default()).unwrap();
    let seeds2 = store.get_home_seeds(HomeSeedLimits::default()).unwrap();
    assert!(!seeds2.continue_listening_track_ids.is_empty());
    assert!(!seeds2.favorite_track_ids.is_empty());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn genre_backfill_makes_top_genres_non_empty() {
    let dir = unique_test_dir("reco-genre-backfill");
    let store = RecoStore::new_at(&dir).unwrap();
    // Plays log no genre_id, so top genres is empty until backfill.
    store.log_play_event(1, Some("jz".into()), Some(10), None).unwrap();
    store.log_play_event(2, Some("jz".into()), Some(10), None).unwrap();
    store.log_play_event(3, Some("rk".into()), Some(11), None).unwrap();
    assert!(store.get_top_genres(10).unwrap().is_empty());
    // The frontend backfills genre on album resolution (id + name).
    store.update_genre_for_album("jz", 5).unwrap();
    store.set_album_genre_name("jz", "Jazz").unwrap();
    store.update_genre_for_album("rk", 6).unwrap();
    store.set_album_genre_name("rk", "Rock").unwrap();
    let genres = store.get_top_genres(10).unwrap();
    assert_eq!(genres.len(), 2);
    assert_eq!(genres[0].0, 5); // jazz played 2x -> ranked first
    assert_eq!(genres[0].1, "Jazz"); // name resolved from album-meta
    assert_eq!(genres[1].0, 6);
    let _ = std::fs::remove_dir_all(dir);
}
