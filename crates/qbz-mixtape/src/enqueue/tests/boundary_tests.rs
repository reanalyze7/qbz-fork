use crate::enqueue::{next_item_index, previous_item_index};
use qbz_models::QueueTrack as CoreQueueTrack;

fn qt(album: &str, item: Option<&str>) -> CoreQueueTrack {
    CoreQueueTrack {
        id: 0,
        title: "t".into(),
        version: None,
        artist: "a".into(),
        album: "alb".into(),
        album_version: None,
        duration_secs: 100,
        artwork_url: None,
        hires: false,
        bit_depth: Some(16),
        sample_rate: Some(44.1),
        is_local: false,
        album_id: Some(album.into()),
        artist_id: None,
        streamable: true,
        source: Some("qobuz".into()),
        parental_warning: false,
        source_item_id_hint: item.map(String::from),
        context_kind: None,
        context_id: None,
    }
}

#[test]
fn next_item_jumps_past_same_hint() {
    let q = vec![
        qt("a1", Some("hint-a1")),
        qt("a1", Some("hint-a1")),
        qt("a2", Some("hint-a2")),
    ];
    assert_eq!(next_item_index(&q, 0), Some(2));
    assert_eq!(next_item_index(&q, 1), Some(2));
    assert_eq!(next_item_index(&q, 2), None);
}

#[test]
fn next_item_falls_back_to_album_id() {
    let q = vec![qt("a1", None), qt("a2", None)];
    assert_eq!(next_item_index(&q, 0), Some(1));
}

#[test]
fn previous_item_restarts_when_mid_item() {
    let q = vec![
        qt("a1", Some("h1")),
        qt("a1", Some("h1")),
        qt("a2", Some("h2")),
    ];
    // current=1 (mid-item of h1), elapsed=500ms → restart at item start (0)
    assert_eq!(previous_item_index(&q, 1, 500), Some(0));
    // current=0 (at item start of h1), elapsed=500ms → same item start, go to prev (0)
    assert_eq!(previous_item_index(&q, 0, 500), Some(0));
    // current=2 (start of h2), elapsed=500ms → go to previous item start (0)
    assert_eq!(previous_item_index(&q, 2, 500), Some(0));
    // current=2 (start of h2), elapsed=5000ms → restart current item (2)
    assert_eq!(previous_item_index(&q, 2, 5_000), Some(2));
}
