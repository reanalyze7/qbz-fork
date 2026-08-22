//! Basic metadata-grouping tests: cross-folder merge, folder fallback,
//! orphan bucket, and Various-Artists detection.

use super::common::{fresh_db, insert_track_for_test};

#[test]
fn metadata_group_merges_tracks_across_folders_with_same_album() {
    let (_tmp, db) = fresh_db();
    // Two folders, same album metadata -> one metadata group.
    insert_track_for_test(
        &db,
        "/m/Bjork/Vespertine/01.flac",
        Some("Vespertine"),
        Some("Bjork"),
        "Bjork",
        "/m/Bjork/Vespertine",
    );
    insert_track_for_test(
        &db,
        "/m/Bjork/Vespertine/02.flac",
        Some("Vespertine"),
        Some("Bjork"),
        "Bjork",
        "/m/Bjork/Vespertine",
    );
    insert_track_for_test(
        &db,
        "/m/mix/cd/track-from-vespertine.flac",
        Some("Vespertine"),
        Some("Bjork"),
        "Bjork",
        "/m/mix/cd",
    );

    let albums = db
        .get_albums_metadata_grouped(false, true, false, crate::album_grouping::AlbumGroupMode::Metadata)
        .unwrap();
    let vespertine = albums
        .iter()
        .find(|a| a.title == "Vespertine")
        .expect("Vespertine group");
    assert_eq!(vespertine.track_count, 3);
}

#[test]
fn metadata_group_falls_back_to_folder_when_album_missing() {
    let (_tmp, db) = fresh_db();
    // Empty album tag -> use folder grouping.
    insert_track_for_test(&db, "/m/folder/01.flac", None, None, "A", "/m/folder");
    insert_track_for_test(&db, "/m/folder/02.flac", None, None, "B", "/m/folder");

    let albums = db
        .get_albums_metadata_grouped(false, true, false, crate::album_grouping::AlbumGroupMode::Metadata)
        .unwrap();
    assert_eq!(albums.len(), 1, "single folder fallback group");
    assert_eq!(albums[0].track_count, 2);
    assert_eq!(albums[0].artist, "Various Artists");
}

#[test]
fn metadata_group_orphan_bucket_when_no_album_no_folder() {
    let (_tmp, db) = fresh_db();
    // No album tag AND no folder key -> orphan bucket.
    insert_track_for_test(&db, "/m/ghost/01.flac", None, None, "X", "");
    insert_track_for_test(&db, "/m/ghost/02.flac", None, None, "Y", "");

    let albums = db
        .get_albums_metadata_grouped(false, true, false, crate::album_grouping::AlbumGroupMode::Metadata)
        .unwrap();
    let unknown = albums
        .iter()
        .find(|a| a.title == "Unknown Album")
        .expect("Unknown Album bucket");
    assert_eq!(unknown.track_count, 2);
}

#[test]
fn metadata_group_va_detection() {
    let (_tmp, db) = fresh_db();
    // Same album, different track artists, album_artist set to VA.
    insert_track_for_test(
        &db,
        "/m/comp/01.flac",
        Some("Comp"),
        Some("Various Artists"),
        "A",
        "/m/comp",
    );
    insert_track_for_test(
        &db,
        "/m/comp/02.flac",
        Some("Comp"),
        Some("Various Artists"),
        "B",
        "/m/comp",
    );

    let albums = db
        .get_albums_metadata_grouped(false, true, false, crate::album_grouping::AlbumGroupMode::Metadata)
        .unwrap();
    let comp = albums
        .iter()
        .find(|a| a.title == "Comp")
        .expect("Comp album");
    assert_eq!(comp.track_count, 2);
    assert_eq!(comp.artist, "Various Artists");
}
