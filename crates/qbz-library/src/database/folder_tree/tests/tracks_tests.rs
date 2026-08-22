//! Tests for `list_folder_tracks` (direct children) and
//! `list_folder_tracks_recursive` (all descendants).

use super::common::{fresh_db, insert_at, seed_standard_fixture};

#[test]
fn list_folder_tracks_excludes_subfolder_contents() {
    let (_tmp, db) = fresh_db();
    seed_standard_fixture(&db);

    // /m/A/album1 has direct tracks t1.flac + t2.flac, plus one
    // track in a subfolder (Disc 1/t3.flac). The latter must NOT
    // appear in list_folder_tracks output.
    let tracks = db.list_folder_tracks("/m/A/album1", false).unwrap();
    assert_eq!(tracks.len(), 2, "subfolder tracks must be excluded");
    let titles: Vec<_> = tracks.iter().map(|t| t.title.as_str()).collect();
    assert!(titles.contains(&"Alpha"));
    assert!(titles.contains(&"Beta"));
    assert!(
        !titles.contains(&"Gamma"),
        "Disc 1/t3.flac leaked into direct-children list"
    );

    // Qobuz download must also be excluded from direct tracks.
    assert!(
        !titles.contains(&"QobuzCache"),
        "qobuz_download row leaked into direct-children list"
    );
}

#[test]
fn list_folder_tracks_orders_by_disc_track_title() {
    let (_tmp, db) = fresh_db();
    // Build a small fixture deliberately out of natural sort order.
    // Expected sort: disc ASC, track ASC, title ASC (NOCASE).
    insert_at(&db, "/m/order/disc2-track1.flac", Some(2), Some(1), "D2T1");
    insert_at(&db, "/m/order/disc1-track2.flac", Some(1), Some(2), "D1T2");
    insert_at(&db, "/m/order/disc1-track1-bee.flac", Some(1), Some(1), "Bee");
    insert_at(
        &db,
        "/m/order/disc1-track1-ant.flac",
        Some(1),
        Some(1),
        "ant", // lowercase — NOCASE collation should sort ant < Bee
    );

    let tracks = db.list_folder_tracks("/m/order", false).unwrap();
    let titles: Vec<_> = tracks.iter().map(|track| track.title.clone()).collect();
    assert_eq!(titles, vec!["ant", "Bee", "D1T2", "D2T1"]);
}

#[test]
fn list_folder_tracks_recursive_includes_all_descendants() {
    let (_tmp, db) = fresh_db();
    seed_standard_fixture(&db);

    // /m/A/album1 has direct tracks t1.flac, t2.flac AND a deeper
    // file at /m/A/album1/Disc 1/t3.flac. The recursive listing
    // must return all three.
    let tracks = db.list_folder_tracks_recursive("/m/A/album1", false).unwrap();
    let titles: Vec<_> = tracks.iter().map(|track| track.title.clone()).collect();
    assert_eq!(tracks.len(), 3, "recursive listing must include subfolder tracks");
    assert!(titles.contains(&"Alpha".to_string()));
    assert!(titles.contains(&"Beta".to_string()));
    assert!(titles.contains(&"Gamma".to_string()));

    // Qobuz download under the same parent must NOT appear.
    assert!(
        !titles.contains(&"QobuzCache".to_string()),
        "qobuz_download row leaked into recursive listing"
    );
}

#[test]
fn list_folder_tracks_recursive_orders_by_file_path() {
    let (_tmp, db) = fresh_db();
    // Insert files deliberately out of file_path order — recursive
    // listing must return them sorted ASC by file_path.
    insert_at(&db, "/m/r/zeta.flac", Some(1), Some(1), "Z");
    insert_at(&db, "/m/r/alpha.flac", Some(1), Some(1), "A");
    insert_at(&db, "/m/r/sub/middle.flac", Some(1), Some(1), "M");

    let tracks = db.list_folder_tracks_recursive("/m/r", false).unwrap();
    let paths: Vec<_> = tracks.iter().map(|track| track.file_path.clone()).collect();
    assert_eq!(
        paths,
        vec![
            "/m/r/alpha.flac".to_string(),
            "/m/r/sub/middle.flac".to_string(),
            "/m/r/zeta.flac".to_string(),
        ]
    );
}

#[test]
fn list_folder_tracks_recursive_handles_special_chars_in_path() {
    let (_tmp, db) = fresh_db();
    // Folder containing literal '_' that LIKE would otherwise treat
    // as a single-character wildcard. With escape_like_pattern, the
    // sibling /m/percentXtest must not contaminate the result set.
    insert_at(&db, "/m/percent_test/100%.flac", Some(1), Some(1), "Hundred");
    insert_at(&db, "/m/percent_test/inner/200.flac", Some(1), Some(1), "TwoHundred");
    insert_at(&db, "/m/percentXtest/decoy.flac", Some(1), Some(1), "Decoy");

    let tracks = db.list_folder_tracks_recursive("/m/percent_test", false).unwrap();
    let titles: Vec<_> = tracks.iter().map(|track| track.title.clone()).collect();
    assert_eq!(tracks.len(), 2, "underscore in parent path must be escaped");
    assert!(titles.contains(&"Hundred".to_string()));
    assert!(titles.contains(&"TwoHundred".to_string()));
    assert!(!titles.contains(&"Decoy".to_string()));
}

#[test]
fn list_folder_tracks_recursive_returns_empty_for_unknown_path() {
    // A folder path with no matching descendants must yield an empty
    // Vec rather than an error — frontend treats empty as "nothing
    // to play/queue" and skips the toast.
    let (_tmp, db) = fresh_db();
    let tracks = db.list_folder_tracks_recursive("/m/does/not/exist", false).unwrap();
    assert!(tracks.is_empty());
}
