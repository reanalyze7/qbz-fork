use qbz_models::QueueTrack;

use super::pure::{album_opt, due};

#[test]
fn due_only_when_past_threshold_and_not_yet_scrobbled() {
    assert!(!due(10, Some(120), false)); // not there yet
    assert!(due(120, Some(120), false)); // exactly at threshold
    assert!(due(200, Some(120), false)); // past it
    assert!(!due(200, Some(120), true)); // already scrobbled
    assert!(!due(999, None, false)); // too short to scrobble (no threshold)
}

fn qt(album: &str) -> QueueTrack {
    QueueTrack {
        id: 1,
        title: "Spain".into(),
        version: None,
        artist: "Chick Corea".into(),
        album: album.into(),
        album_version: None,
        duration_secs: 300,
        artwork_url: None,
        hires: false,
        bit_depth: None,
        sample_rate: None,
        is_local: false,
        album_id: None,
        artist_id: None,
        streamable: true,
        source: None,
        parental_warning: false,
        source_item_id_hint: None,
        context_kind: None,
        context_id: None,
    }
}

#[test]
fn album_opt_drops_empty_and_unknown() {
    assert_eq!(album_opt(&qt("Light as a Feather")), Some("Light as a Feather"));
    assert_eq!(album_opt(&qt("")), None);
    assert_eq!(album_opt(&qt("Unknown Album")), None);
}
