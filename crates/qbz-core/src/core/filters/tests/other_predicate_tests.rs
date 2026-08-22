use qbz_models::{DiscoverAlbum, DiscoverAlbumImage, DiscoverArtist};

use super::super::{discover_album_blacklisted, track_blacklisted};
use super::{no_albums, track_with, track_with_album};
use super::super::super::{AlbumBlacklistFilter, BlacklistFilter};

#[test]
fn track_blacklisted_blocks_on_performer() {
    let track = track_with(Some(5), None);
    let bl: BlacklistFilter = [5].into_iter().collect();
    assert!(track_blacklisted(&track, &bl, &no_albums()));
}

#[test]
fn track_blacklisted_blocks_on_composer() {
    let track = track_with(Some(1), Some(7));
    let bl: BlacklistFilter = [7].into_iter().collect();
    assert!(track_blacklisted(&track, &bl, &no_albums()));
}

#[test]
fn track_blacklisted_keeps_when_no_match() {
    let track = track_with(Some(1), Some(2));
    let bl: BlacklistFilter = [999].into_iter().collect();
    assert!(!track_blacklisted(&track, &bl, &no_albums()));
}

#[test]
fn track_blacklisted_fail_open_when_no_ids() {
    // No performer + no composer => kept (fail-open).
    let track = track_with(None, None);
    let bl: BlacklistFilter = [1, 2, 3].into_iter().collect();
    assert!(!track_blacklisted(&track, &bl, &no_albums()));
}

#[test]
fn track_blacklisted_empty_filter_is_false() {
    let track = track_with(Some(5), Some(7));
    let bl: BlacklistFilter = BlacklistFilter::new();
    assert!(!track_blacklisted(&track, &bl, &no_albums()));
}

#[test]
fn discover_album_blacklisted_blocks_on_any_artist() {
    let album = DiscoverAlbum {
        id: String::new(),
        title: String::new(),
        version: None,
        track_count: None,
        duration: None,
        parental_warning: None,
        image: DiscoverAlbumImage {
            small: None,
            thumbnail: None,
            large: None,
        },
        artists: vec![
            DiscoverArtist {
                id: 1,
                name: String::new(),
                roles: None,
            },
            DiscoverArtist {
                id: 999,
                name: String::new(),
                roles: None,
            },
        ],
        label: None,
        genre: None,
        dates: None,
        audio_info: None,
    };
    let blocked: BlacklistFilter = [999].into_iter().collect();
    assert!(discover_album_blacklisted(&album, &blocked, &no_albums()));
    let kept: BlacklistFilter = [555].into_iter().collect();
    assert!(!discover_album_blacklisted(&album, &kept, &no_albums()));
}

#[test]
fn track_blocked_by_album_id() {
    let track = track_with_album("blk");
    let abl: AlbumBlacklistFilter = ["blk".to_string()].into_iter().collect();
    assert!(track_blacklisted(&track, &BlacklistFilter::new(), &abl));
    // No album / different id => kept.
    assert!(!track_blacklisted(
        &track_with(Some(1), None),
        &BlacklistFilter::new(),
        &abl
    ));
}

#[test]
fn discover_album_blocked_by_own_id() {
    let mut album = DiscoverAlbum {
        id: "blk".to_string(),
        title: String::new(),
        version: None,
        track_count: None,
        duration: None,
        parental_warning: None,
        image: DiscoverAlbumImage {
            small: None,
            thumbnail: None,
            large: None,
        },
        artists: vec![DiscoverArtist {
            id: 1,
            name: String::new(),
            roles: None,
        }],
        label: None,
        genre: None,
        dates: None,
        audio_info: None,
    };
    let abl: AlbumBlacklistFilter = ["blk".to_string()].into_iter().collect();
    assert!(discover_album_blacklisted(&album, &BlacklistFilter::new(), &abl));
    album.id = "other".to_string();
    assert!(!discover_album_blacklisted(&album, &BlacklistFilter::new(), &abl));
}
