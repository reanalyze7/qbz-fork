//! Metadata-grouping edge cases: fetching tracks for a group key, and
//! the #507 album-artist-vs-mixed-track-artists case.

use super::common::{fresh_db, insert_full_track_for_test, insert_track_for_test};

#[test]
fn metadata_group_tracks_fetch_returns_all_in_group() {
    let (_tmp, db) = fresh_db();
    insert_track_for_test(
        &db,
        "/m/folderA/01.flac",
        Some("Album X"),
        Some("Artist Y"),
        "Artist Y",
        "/m/folderA",
    );
    insert_track_for_test(
        &db,
        "/m/folderB/02.flac",
        Some("Album X"),
        Some("Artist Y"),
        "Artist Y",
        "/m/folderB",
    );
    insert_track_for_test(
        &db,
        "/m/folderA/03.flac",
        Some("Album X"),
        Some("Artist Y"),
        "Artist Y",
        "/m/folderA",
    );
    // Different album in same folder set
    insert_track_for_test(
        &db,
        "/m/folderA/04.flac",
        Some("Album Z"),
        Some("Artist Y"),
        "Artist Y",
        "/m/folderA",
    );

    let key = "Album X|Artist Y";
    let tracks = db.get_album_tracks_metadata(key).unwrap();
    assert_eq!(tracks.len(), 3);
}

#[test]
fn metadata_group_respects_album_artist_over_mixed_track_artists() {
    // #507 core: every track carries the same Album Artist tag while the
    // per-track artists differ -> the album shows the album artist, NOT
    // "Various Artists".
    let (_tmp, db) = fresh_db();
    insert_full_track_for_test(
        &db,
        "/m/mix/t1.flac",
        "Mix Album",
        Some("Curated Artist"),
        "Artist A",
        "/m/mix",
        "mix",
        Some(2025),
    );
    insert_full_track_for_test(
        &db,
        "/m/mix/t2.flac",
        "Mix Album",
        Some("Curated Artist"),
        "Artist B",
        "/m/mix",
        "mix",
        Some(2025),
    );

    let albums = db
        .get_albums_metadata_grouped(false, true, false, crate::album_grouping::AlbumGroupMode::Metadata)
        .unwrap();
    let mix = albums
        .iter()
        .find(|a| a.title == "Mix Album")
        .expect("Mix Album group");
    assert_eq!(mix.artist, "Curated Artist");
}
