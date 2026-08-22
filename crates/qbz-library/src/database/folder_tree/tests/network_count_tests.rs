//! Tests for `count_folder_tracks_recursive` — the lightweight COUNT
//! companion to `list_folder_tracks_recursive`. See `network_tests`
//! for the listing-primitive network-exclude tests.

use super::common::{fresh_db, insert_at, seed_standard_fixture};

/// `count_folder_tracks_recursive` mirrors
/// `list_folder_tracks_recursive` row-for-row: same source filter
/// (qobuz_download excluded), same network-folder NOT EXISTS
/// predicate, same prefix-with-slash boundary. The count is what
/// the tree-mode rail uses for top-level scan-root rows that don't
/// come back through `list_folder_children`.
#[test]
fn count_folder_tracks_recursive_matches_listing_primitive() {
    let (_tmp, db) = fresh_db();
    seed_standard_fixture(&db);

    // /m/A/album1 has 3 user descendants (t1, t2, Disc 1/t3) and one
    // qobuz_download (qcache.flac) under the same parent. Recursive
    // count must include the deeper subfolder track AND exclude the
    // qobuz row.
    let count = db
        .count_folder_tracks_recursive("/m/A/album1", false)
        .unwrap();
    assert_eq!(count, 3);

    // /m as the root catches every user track in the fixture.
    let total = db.count_folder_tracks_recursive("/m", false).unwrap();
    assert_eq!(total, 5);

    // Unknown path returns 0 (no rows match the LIKE prefix).
    let empty = db
        .count_folder_tracks_recursive("/m/does/not/exist", false)
        .unwrap();
    assert_eq!(empty, 0);
}

/// Network-folder filter must apply to the count primitive too —
/// otherwise the rail would advertise tracks that don't show up in
/// the listing, recursive playback, or flat-mode search.
#[test]
fn count_folder_tracks_recursive_honors_network_exclude() {
    let (_tmp, db) = fresh_db();

    db.add_folder_with_network_info("/m/local", false, None)
        .unwrap();
    db.add_folder_with_network_info("/m/net", true, Some("nfs"))
        .unwrap();

    insert_at(&db, "/m/local/album/local1.flac", Some(1), Some(1), "L1");
    insert_at(&db, "/m/local/album/local2.flac", Some(1), Some(2), "L2");
    insert_at(&db, "/m/net/album/net1.flac", Some(1), Some(1), "N1");
    insert_at(&db, "/m/net/album/sub/net2.flac", Some(1), Some(1), "N2");

    // Without filter: every descendant counts.
    assert_eq!(
        db.count_folder_tracks_recursive("/m/net", false).unwrap(),
        2
    );
    assert_eq!(
        db.count_folder_tracks_recursive("/m/local", false).unwrap(),
        2
    );

    // With filter: network root collapses to 0, local stays.
    assert_eq!(db.count_folder_tracks_recursive("/m/net", true).unwrap(), 0);
    assert_eq!(
        db.count_folder_tracks_recursive("/m/local", true).unwrap(),
        2
    );
}
