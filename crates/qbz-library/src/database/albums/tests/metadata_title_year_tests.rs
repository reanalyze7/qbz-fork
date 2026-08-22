//! The #447 title/year-per-album regressions: the live album tag must
//! win over the scan-time folder-name snapshot, and each tagged album
//! sharing a folder must keep its own `year`.

use super::common::{fresh_db, insert_full_track_for_test};

#[test]
fn metadata_group_title_prefers_album_tag_over_folder_snapshot() {
    // #447 title: the live album tag differs from the scan-time
    // album_group_title snapshot (folder name) -> the tag wins.
    let (_tmp, db) = fresh_db();
    insert_full_track_for_test(
        &db,
        "/m/Alle Songs/t1.flac",
        "ALBUM.",
        Some("The Artist"),
        "The Artist",
        "/m/Alle Songs",
        "Alle Songs",
        Some(2025),
    );

    let albums = db
        .get_albums_metadata_grouped(false, true, false, crate::album_grouping::AlbumGroupMode::Metadata)
        .unwrap();
    let a = albums
        .iter()
        .find(|a| a.title == "ALBUM.")
        .expect("album tag title");
    assert_eq!(a.title, "ALBUM.");
}

#[test]
fn metadata_group_year_is_per_album_not_per_folder() {
    // #447 year: two tagged albums sharing one folder must split into
    // two metadata groups, each with its OWN year — a folder-level
    // group would MIN() them together and show the oldest year for both.
    let (_tmp, db) = fresh_db();
    insert_full_track_for_test(
        &db,
        "/m/Alle Songs/old.flac",
        "Old Album",
        Some("X"),
        "X",
        "/m/Alle Songs",
        "Alle Songs",
        Some(2004),
    );
    insert_full_track_for_test(
        &db,
        "/m/Alle Songs/new1.flac",
        "New Album",
        Some("X"),
        "X",
        "/m/Alle Songs",
        "Alle Songs",
        Some(2025),
    );
    insert_full_track_for_test(
        &db,
        "/m/Alle Songs/new2.flac",
        "New Album",
        Some("X"),
        "X",
        "/m/Alle Songs",
        "Alle Songs",
        Some(2025),
    );

    let albums = db
        .get_albums_metadata_grouped(false, true, false, crate::album_grouping::AlbumGroupMode::Metadata)
        .unwrap();
    let old = albums
        .iter()
        .find(|a| a.title == "Old Album")
        .expect("Old Album group");
    let new = albums
        .iter()
        .find(|a| a.title == "New Album")
        .expect("New Album group");
    assert_eq!(old.year, Some(2004));
    assert_eq!(new.year, Some(2025));
    assert_eq!(new.track_count, 2);
}
