use qbz_models::Album;

use super::super::album_blacklisted;
use super::{album_with_artists, album_with_id, no_albums};
use super::super::super::{AlbumBlacklistFilter, BlacklistFilter};

#[test]
fn album_blacklisted_blocks_on_primary_artist() {
    let album = album_with_artists(1, &[]);
    let bl: BlacklistFilter = [1].into_iter().collect();
    assert!(album_blacklisted(&album, &bl, &no_albums()));
}

#[test]
fn album_blacklisted_blocks_on_featured_not_primary() {
    // Primary is 1 (kept), featured 999 is blocked.
    let album = album_with_artists(1, &[999]);
    let bl: BlacklistFilter = [999].into_iter().collect();
    assert!(album_blacklisted(&album, &bl, &no_albums()));
}

#[test]
fn album_blacklisted_keeps_when_no_match() {
    let album = album_with_artists(1, &[2, 3]);
    let bl: BlacklistFilter = [999].into_iter().collect();
    assert!(!album_blacklisted(&album, &bl, &no_albums()));
}

#[test]
fn album_blacklisted_empty_filter_is_false() {
    let album = album_with_artists(1, &[999]);
    let bl: BlacklistFilter = BlacklistFilter::new();
    assert!(!album_blacklisted(&album, &bl, &no_albums()));
}

#[test]
fn album_blocked_by_own_id_regardless_of_artist() {
    // Artist filter EMPTY; only the album id is blocked. The widened
    // fail-open guard must NOT early-return here.
    let album = album_with_id("blk", 1);
    let abl: AlbumBlacklistFilter = ["blk".to_string()].into_iter().collect();
    assert!(album_blacklisted(&album, &BlacklistFilter::new(), &abl));
    let other: AlbumBlacklistFilter = ["zzz".to_string()].into_iter().collect();
    assert!(!album_blacklisted(&album, &BlacklistFilter::new(), &other));
}

#[test]
fn album_blocked_keeps_sibling_album_of_same_artist() {
    // The merged-artist use case: blocking one album by id leaves the same
    // artist's other releases visible.
    let blocked_album = album_with_id("bad", 1);
    let good_album: Album = album_with_id("good", 1);
    let abl: AlbumBlacklistFilter = ["bad".to_string()].into_iter().collect();
    assert!(album_blacklisted(&blocked_album, &BlacklistFilter::new(), &abl));
    assert!(!album_blacklisted(&good_album, &BlacklistFilter::new(), &abl));
}
