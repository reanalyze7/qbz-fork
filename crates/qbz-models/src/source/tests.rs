use super::*;
use crate::playback::QueueTrack;

#[test]
fn source_str_roundtrip_and_default() {
    assert_eq!(PlaybackSource::from_source_str(Some("local")), PlaybackSource::Local);
    assert_eq!(
        PlaybackSource::from_source_str(Some("qobuz_download")),
        PlaybackSource::OfflineCache
    );
    assert_eq!(PlaybackSource::from_source_str(Some("qobuz")), PlaybackSource::Qobuz);
    // Unknown / absent -> Qobuz (historical default).
    assert_eq!(PlaybackSource::from_source_str(None), PlaybackSource::Qobuz);
    assert_eq!(PlaybackSource::from_source_str(Some("???")), PlaybackSource::Qobuz);
    for s in [PlaybackSource::Qobuz, PlaybackSource::OfflineCache, PlaybackSource::Local] {
        assert_eq!(PlaybackSource::from_source_str(Some(s.as_source_str())), s);
    }
}

#[test]
fn offline_cache_is_castable() {
    assert!(PlaybackSource::OfflineCache.is_castable_to_qconnect());
    assert!(TrackOriginTag::OfflineCache.is_castable_to_qconnect());
}

#[test]
fn strict_parse_blocks_unknown_and_absent() {
    use TrackOriginTag::*;
    assert_eq!(PlaybackSource::from_source_str_strict(Some("qobuz")), Qobuz);
    assert_eq!(PlaybackSource::from_source_str_strict(Some("local")), Local);
    assert_eq!(PlaybackSource::from_source_str_strict(Some("qobuz_download")), OfflineCache);
    assert_eq!(PlaybackSource::from_source_str_strict(None), ExternalUnknown);
    assert_eq!(PlaybackSource::from_source_str_strict(Some("???")), ExternalUnknown);
    // Lenient parser still defaults to Qobuz (playback compatibility).
    assert_eq!(PlaybackSource::from_source_str(None), PlaybackSource::Qobuz);
}

#[test]
fn only_qobuz_is_castable() {
    assert!(PlaybackSource::Qobuz.is_qobuz_streamable());
    assert!(!PlaybackSource::OfflineCache.is_qobuz_streamable());
    assert!(!PlaybackSource::Local.is_qobuz_streamable());
}

fn track_with(source: Option<&str>, artwork: Option<&str>) -> QueueTrack {
    QueueTrack {
        id: 1,
        title: "t".into(),
        version: None,
        artist: "a".into(),
        album: "al".into(),
        album_version: None,
        duration_secs: 0,
        artwork_url: artwork.map(|s| s.to_string()),
        hires: false,
        bit_depth: None,
        sample_rate: None,
        is_local: false,
        album_id: None,
        artist_id: None,
        streamable: true,
        source: source.map(|s| s.to_string()),
        parental_warning: false,
        source_item_id_hint: None,
        context_kind: None,
        context_id: None,
    }
}

#[test]
fn artwork_ref_classifies_by_value() {
    assert_eq!(track_with(None, None).artwork_ref(), ArtworkRef::None);
    assert_eq!(
        track_with(Some("qobuz"), Some("https://x/cover.jpg")).artwork_ref(),
        ArtworkRef::Remote("https://x/cover.jpg".into())
    );
    assert_eq!(
        track_with(Some("local"), Some("/home/u/cover.jpg")).artwork_ref(),
        ArtworkRef::LocalFile("/home/u/cover.jpg".into())
    );
    assert_eq!(
        track_with(Some("local"), Some("file:///home/u/cover.jpg")).artwork_ref(),
        ArtworkRef::LocalFile("/home/u/cover.jpg".into())
    );
}
