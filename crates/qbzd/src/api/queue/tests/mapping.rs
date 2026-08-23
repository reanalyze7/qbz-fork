use qbz_models::Track;

use crate::api::queue::mapping::track_to_queue_track;

#[test]
fn track_to_queue_track_maps_the_catalog_shape() {
    let mut track = Track {
        id: 176544872,
        title: "500 Miles High".into(),
        duration: 547,
        hires: true,
        maximum_bit_depth: Some(24),
        maximum_sampling_rate: Some(96.0),
        streamable: true,
        parental_warning: false,
        ..Default::default()
    };
    track.performer = Some(qbz_models::Artist {
        id: 123206,
        name: "Chick Corea".into(),
        ..Default::default()
    });
    track.album = Some(qbz_models::AlbumSummary {
        id: "0060253776847".into(),
        title: "Light as a Feather".into(),
        image: qbz_models::ImageSet {
            large: Some("https://static.qobuz.com/large.jpg".into()),
            ..Default::default()
        },
        label: None,
        genre: None,
    });

    let qt = track_to_queue_track(&track);
    assert_eq!(qt.id, 176544872);
    assert_eq!(qt.title, "500 Miles High");
    assert_eq!(qt.artist, "Chick Corea");
    assert_eq!(qt.album, "Light as a Feather");
    assert_eq!(qt.duration_secs, 547);
    assert_eq!(qt.album_id.as_deref(), Some("0060253776847"));
    assert_eq!(qt.artist_id, Some(123206));
    assert_eq!(qt.source.as_deref(), Some("qobuz"));
    assert_eq!(qt.artwork_url.as_deref(), Some("https://static.qobuz.com/large.jpg"));
}

#[test]
fn track_to_queue_track_falls_back_when_performer_and_album_are_absent() {
    let track = Track {
        id: 1,
        title: "Untitled".into(),
        ..Default::default()
    };
    let qt = track_to_queue_track(&track);
    assert_eq!(qt.artist, "Unknown Artist");
    assert_eq!(qt.album, "Unknown Album");
    assert!(qt.album_id.is_none());
    assert!(qt.artwork_url.is_none());
}
