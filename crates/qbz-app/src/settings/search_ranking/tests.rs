use super::*;
use std::path::PathBuf;

fn unique_test_dir(name: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("qbz_search_ranking_{name}_{nonce}"))
}

#[test]
fn record_bumps_by_correct_weights_and_accumulates() {
    let dir = unique_test_dir("weights");
    let mut r = SearchRanking::new(&dir);
    r.record("Pink Floyd", "artist", "42", InteractionAction::Open); // +1
    assert_eq!(r.score_for("pink floyd", "artist", "42"), 1);
    r.record("Pink Floyd", "artist", "42", InteractionAction::Play); // +2
    assert_eq!(r.score_for("Pink Floyd", "artist", "42"), 3);
    r.record("Pink Floyd", "artist", "42", InteractionAction::Favorite); // +3
    assert_eq!(r.score_for("PINK FLOYD", "artist", "42"), 6);
    // Distinct entity is tracked separately.
    r.record("Pink Floyd", "album", "99", InteractionAction::Play);
    assert_eq!(r.score_for("pink floyd", "album", "99"), 2);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn top_for_query_returns_the_max() {
    let dir = unique_test_dir("top");
    let mut r = SearchRanking::new(&dir);
    r.record("daft punk", "artist", "1", InteractionAction::Open); // 1
    r.record("daft punk", "album", "2", InteractionAction::Favorite); // 3
    r.record("daft punk", "track", "3", InteractionAction::Play); // 2
    assert_eq!(
        r.top_for_query("Daft Punk"),
        Some(("album".to_string(), "2".to_string()))
    );
    assert_eq!(r.top_for_query("never searched"), None);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn score_cap_holds() {
    let dir = unique_test_dir("cap");
    let mut r = SearchRanking::new(&dir);
    // 500 favorites * 3 = 1500, capped at MAX_SCORE.
    for _ in 0..500 {
        r.record("x", "track", "7", InteractionAction::Favorite);
    }
    assert_eq!(r.score_for("x", "track", "7"), MAX_SCORE);
    // Further bumps don't exceed the cap.
    r.record("x", "track", "7", InteractionAction::Play);
    assert_eq!(r.score_for("x", "track", "7"), MAX_SCORE);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn query_lru_cap_evicts_oldest() {
    let dir = unique_test_dir("lru");
    let mut r = SearchRanking::new(&dir);
    // Fill exactly to the cap.
    for i in 0..MAX_QUERIES {
        r.record(&format!("q{i}"), "artist", "1", InteractionAction::Open);
    }
    assert_eq!(r.ranking.len(), MAX_QUERIES);
    // q0 is the oldest, still present.
    assert_eq!(r.score_for("q0", "artist", "1"), 1);
    // One more distinct query evicts the oldest (q0).
    r.record("overflow", "artist", "1", InteractionAction::Open);
    assert_eq!(r.ranking.len(), MAX_QUERIES);
    assert_eq!(r.score_for("q0", "artist", "1"), 0); // evicted
    assert_eq!(r.score_for("overflow", "artist", "1"), 1); // present
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn persistence_round_trips() {
    let dir = unique_test_dir("persist");
    {
        let mut r = SearchRanking::new(&dir);
        r.record("radiohead", "album", "okc", InteractionAction::Favorite); // 3
        r.record("radiohead", "track", "creep", InteractionAction::Play); // 2
    }
    // Fresh instance over the SAME dir reads persisted state.
    let r2 = SearchRanking::new(&dir);
    assert_eq!(r2.score_for("radiohead", "album", "okc"), 3);
    assert_eq!(r2.score_for("radiohead", "track", "creep"), 2);
    assert_eq!(
        r2.top_for_query("Radiohead"),
        Some(("album".to_string(), "okc".to_string()))
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn corrupt_file_loads_empty() {
    let dir = unique_test_dir("corrupt");
    let search_dir = dir.join("search");
    std::fs::create_dir_all(&search_dir).unwrap();
    std::fs::write(search_dir.join("search_ranking.json"), b"{ not json").unwrap();
    let r = SearchRanking::new(&dir);
    assert_eq!(r.score_for("anything", "artist", "1"), 0);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn rank_within_reorders_scored_ahead_keeping_unscored_stable() {
    let dir = unique_test_dir("rank");
    let mut r = SearchRanking::new(&dir);
    // Learn scores: id "b" highest, id "d" lower, others unseen.
    r.record("metallica", "album", "b", InteractionAction::Favorite); // 3
    r.record("metallica", "album", "d", InteractionAction::Open); // 1

    // Original API order: a, b, c, d, e (a/c/e unscored).
    let mut items = vec!["a", "b", "c", "d", "e"];
    r.rank_within("metallica", "album", &mut items, |s| s.to_string());

    // Scored items first (b=3, d=1), then unscored in original order (a,c,e).
    assert_eq!(items, vec!["b", "d", "a", "c", "e"]);

    // A query with nothing learned leaves order untouched.
    let mut untouched = vec!["x", "y", "z"];
    r.rank_within("unknown", "album", &mut untouched, |s| s.to_string());
    assert_eq!(untouched, vec!["x", "y", "z"]);
    let _ = std::fs::remove_dir_all(&dir);
}
