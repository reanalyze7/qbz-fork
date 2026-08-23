use super::selector::{parse_selector, Selector};
use super::util::clamp_index;

#[test]
fn parse_selector_reads_each_id_field() {
    assert!(matches!(
        parse_selector(&serde_json::json!({"track_id": 42})).unwrap(),
        Selector::Track(42)
    ));
    assert!(matches!(
        parse_selector(&serde_json::json!({"album_id": "abc"})).unwrap(),
        Selector::Album(ref s) if s == "abc"
    ));
    assert!(matches!(
        parse_selector(&serde_json::json!({"playlist_id": 7})).unwrap(),
        Selector::Playlist(7)
    ));
    assert!(matches!(
        parse_selector(&serde_json::json!({"artist_id": 9})).unwrap(),
        Selector::Artist(9)
    ));
}

#[test]
fn parse_selector_resolves_a_qobuz_url_and_it_wins_over_ids() {
    // URL resolves to a kind, and it takes precedence over a stray id field.
    let body = serde_json::json!({
        "url": "https://open.qobuz.com/album/0060254728933",
        "track_id": 42
    });
    assert!(matches!(
        parse_selector(&body).unwrap(),
        Selector::Album(ref s) if s == "0060254728933"
    ));
}

#[test]
fn parse_selector_rejects_a_bad_url_and_a_missing_selector() {
    assert!(parse_selector(&serde_json::json!({"url": "https://example.com/x"})).is_err());
    assert!(parse_selector(&serde_json::json!({})).is_err());
}

#[test]
fn clamp_index_defaults_zero_and_clamps_into_range() {
    assert_eq!(clamp_index(None, 5), 0);
    assert_eq!(clamp_index(Some(2), 5), 2);
    assert_eq!(clamp_index(Some(99), 5), 4);
    assert_eq!(clamp_index(Some(0), 0), 0);
}
