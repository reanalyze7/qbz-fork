//! Folder-grouping-mode regression test (spec 2026-07-19
//! local-album-grouping-mode §D — the "Saint Seiya" compilation case).

use super::common::{fresh_db, insert_track_for_test};

#[test]
fn folder_group_mode_compilation_is_one_album() {
    let (_tmp, db) = fresh_db();
    // Saint Seiya case: one folder, same album tag, 10 distinct track
    // artists, NO album_artist. Metadata mode splits per track artist;
    // Folder mode keeps ONE card.
    for (i, artist) in [
        "MAKE-UP", "MAKE-UP PROJECT", "Horie", "Kageyama", "Furuya",
        "Trooper", "Matsuzawa", "Marina", "Broadway", "Oren",
    ]
    .iter()
    .enumerate()
    {
        insert_track_for_test(
            &db,
            &format!("/m/seiya/{:02}.flac", i + 1),
            Some("Saint Seiya Best"),
            None,
            artist,
            "/m/seiya",
        );
    }

    // Metadata mode: one group per album|artist pair (the #411 split).
    let albums = db
        .get_albums_metadata_grouped(false, true, false, crate::album_grouping::AlbumGroupMode::Metadata)
        .unwrap();
    assert_eq!(albums.len(), 10, "metadata mode splits per track artist");

    // Folder mode: ONE album, Various Artists, everyone in all_artists.
    let albums = db
        .get_albums_metadata_grouped(false, true, false, crate::album_grouping::AlbumGroupMode::Folder)
        .unwrap();
    assert_eq!(albums.len(), 1, "folder mode keeps the compilation whole");
    let comp = &albums[0];
    assert_eq!(comp.title, "Saint Seiya Best");
    assert_eq!(comp.artist, "Various Artists");
    assert_eq!(comp.track_count, 10);
    let all = comp.all_artists.as_str();
    for artist in ["MAKE-UP", "Horie", "Kageyama", "Marina"] {
        assert!(all.contains(artist), "all_artists carries {artist}");
    }

    // Same folder with album_artist set -> that artist, not VA.
    let (_tmp2, db2) = fresh_db();
    insert_track_for_test(
        &db2,
        "/m/eels/01.flac",
        Some("Beautiful Freak"),
        Some("EELS"),
        "EELS",
        "/m/eels",
    );
    insert_track_for_test(
        &db2,
        "/m/eels/02.flac",
        Some("Beautiful Freak"),
        Some("EELS"),
        "EELS",
        "/m/eels",
    );
    let albums = db2
        .get_albums_metadata_grouped(false, true, false, crate::album_grouping::AlbumGroupMode::Folder)
        .unwrap();
    assert_eq!(albums.len(), 1);
    assert_eq!(albums[0].artist, "EELS");

    // Orphan bucket still works in folder mode (no folder key at all).
    let (_tmp3, db3) = fresh_db();
    insert_track_for_test(&db3, "/m/ghost/01.flac", None, None, "X", "");
    insert_track_for_test(&db3, "/m/ghost/02.flac", None, None, "Y", "");
    let albums = db3
        .get_albums_metadata_grouped(false, true, false, crate::album_grouping::AlbumGroupMode::Folder)
        .unwrap();
    let unknown = albums
        .iter()
        .find(|a| a.title == "Unknown Album")
        .expect("orphan bucket in folder mode");
    assert_eq!(unknown.track_count, 2);
}
