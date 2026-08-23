use qbz_models::QueueTrack;

pub(super) fn sample_track(id: u64) -> QueueTrack {
    QueueTrack {
        id,
        title: format!("Track {id}"),
        version: None,
        artist: "Chick Corea".into(),
        album: "Light as a Feather".into(),
        album_version: None,
        duration_secs: 300,
        artwork_url: None,
        hires: true,
        bit_depth: Some(24),
        sample_rate: Some(96.0),
        is_local: false,
        album_id: Some("0060253776847".into()),
        artist_id: Some(123206),
        streamable: true,
        source: Some("qobuz".into()),
        parental_warning: false,
        source_item_id_hint: None,
        context_kind: None,
        context_id: None,
    }
}
