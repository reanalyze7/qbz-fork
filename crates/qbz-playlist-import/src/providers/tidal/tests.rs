use super::duration::parse_duration_ms;
use super::url::{detect_resource, parse_playlist_id};
use crate::providers::MusicResource;

#[test]
fn parse_playlist_id_table() {
    let cases: &[(&str, Option<&str>)] = &[
        (
            "https://tidal.com/browse/playlist/1b418bb8-90a7-4f87-901d-707993838346",
            Some("1b418bb8-90a7-4f87-901d-707993838346"),
        ),
        (
            "https://listen.tidal.com/playlist/1b418bb8-90a7-4f87-901d-707993838346",
            Some("1b418bb8-90a7-4f87-901d-707993838346"),
        ),
        ("https://tidal.com/playlist/abc?u=1", Some("abc")),
        ("https://tidal.com/browse/album/123", None),
        ("https://tidal.com/playlist/", None),
        ("https://example.com/playlist/abc", None),
    ];

    for (url, expected) in cases {
        assert_eq!(parse_playlist_id(url).as_deref(), *expected, "url: {}", url);
    }
}

#[test]
fn detect_resource_track_album_playlist() {
    assert!(matches!(
        detect_resource("https://tidal.com/browse/playlist/abc"),
        Some(MusicResource::Playlist { .. })
    ));
    assert!(matches!(
        detect_resource("https://tidal.com/browse/track/12345"),
        Some(MusicResource::Track { .. })
    ));
    assert!(matches!(
        detect_resource("https://tidal.com/album/67890"),
        Some(MusicResource::Album { .. })
    ));
    assert_eq!(detect_resource("https://example.com/track/1"), None);
}

#[test]
fn parse_duration_ms_iso8601_forms() {
    assert_eq!(parse_duration_ms("PT3M21S"), Some(201_000));
    assert_eq!(parse_duration_ms("PT1H2M3S"), Some(3_723_000));
    assert_eq!(parse_duration_ms("PT45S"), Some(45_000));
    // No leading P
    assert_eq!(parse_duration_ms("3M21S"), None);
    // No digit/unit pairs at all
    assert_eq!(parse_duration_ms("PT"), None);
    assert_eq!(parse_duration_ms(""), None);
}
