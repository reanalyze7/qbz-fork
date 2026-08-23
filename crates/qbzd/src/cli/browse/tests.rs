use serde_json::Value;

use super::render::collect_ids;
use super::{render, to_similar_query};

#[test]
fn to_similar_query_builds_artist_and_album_paths() {
    assert_eq!(to_similar_query("artist:123", 10).unwrap(), "/api/similar?artist=123&limit=10");
    assert_eq!(to_similar_query("album:abc", 5).unwrap(), "/api/similar?album=abc&limit=5");
    assert!(to_similar_query("artist:xy", 10).is_err());
    assert!(to_similar_query("nope", 10).is_err());
}

#[test]
fn walk_collects_items_and_top_level_tracks_only() {
    // album shape: album.tracks.items ; plus a nested track.album (must NOT
    // be collected — it is not under an items/tracks array key).
    let album = serde_json::json!({
        "album": {"id": "A", "title": "Al",
            "tracks": {"items": [
                {"id": 1, "title": "T1", "album": {"id": "A", "title": "Al"}},
                {"id": 2, "title": "T2"}
            ]}}
    });
    assert_eq!(collect_ids(&album), vec!["1", "2"]);

    // suggest shape: top-level tracks array of Track.
    let suggest = serde_json::json!({"tracks": [{"id": 9, "title": "S"}]});
    assert_eq!(collect_ids(&suggest), vec!["9"]);
}

#[test]
fn render_empty_says_no_results() {
    assert_eq!(render(&serde_json::json!({"album": Value::Null})), "no results\n");
}
