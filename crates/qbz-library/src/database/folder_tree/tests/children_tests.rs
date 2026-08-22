//! Tests for `list_folder_children`: ordering, Qobuz-download
//! filtering, and LIKE-pattern special-character escaping.

use crate::FolderTreeEntry;

use super::common::{fresh_db, insert_at, seed_standard_fixture};

#[test]
fn list_folder_children_returns_folders_before_tracks() {
    let (_tmp, db) = fresh_db();
    seed_standard_fixture(&db);

    // /m/A/album1 has: subfolder "Disc 1", tracks "t1.flac" + "t2.flac".
    // Expected order: folder first, then tracks alphabetical.
    let children = db.list_folder_children("/m/A/album1", false).unwrap();
    assert_eq!(children.len(), 3, "one folder + two tracks expected");

    match &children[0] {
        FolderTreeEntry::Folder {
            segment,
            track_count_under,
            path,
            ..
        } => {
            assert_eq!(segment, "Disc 1");
            assert_eq!(*track_count_under, 1);
            assert_eq!(path, "/m/A/album1/Disc 1");
        }
        other => panic!("expected folder first, got {:?}", other),
    }
    match &children[1] {
        FolderTreeEntry::Track { segment, path } => {
            assert_eq!(segment, "t1.flac");
            assert_eq!(path, "/m/A/album1/t1.flac");
        }
        other => panic!("expected track at index 1, got {:?}", other),
    }
    match &children[2] {
        FolderTreeEntry::Track { segment, .. } => {
            assert_eq!(segment, "t2.flac");
        }
        other => panic!("expected track at index 2, got {:?}", other),
    }
}

#[test]
fn list_folder_children_filters_qobuz_downloads() {
    let (_tmp, db) = fresh_db();
    seed_standard_fixture(&db);

    let children = db.list_folder_children("/m/A/album1", false).unwrap();
    // qcache.flac (qobuz_download) must not appear, even though it
    // shares the same parent folder as t1.flac/t2.flac.
    for entry in &children {
        if let FolderTreeEntry::Track { segment, .. } = entry {
            assert_ne!(
                segment, "qcache.flac",
                "qobuz_download row leaked into tree"
            );
        }
    }

    // Track count under "Disc 1" should also exclude any qobuz rows
    // (none here, but the filter must hold even at folder level).
    let folder_count = children
        .iter()
        .filter_map(|e| match e {
            FolderTreeEntry::Folder {
                track_count_under, ..
            } => Some(*track_count_under),
            _ => None,
        })
        .sum::<u32>();
    assert_eq!(folder_count, 1, "Disc 1 contains exactly 1 user track");
}

#[test]
fn list_folder_children_handles_special_chars_in_path() {
    let (_tmp, db) = fresh_db();
    seed_standard_fixture(&db);

    // Folder containing a literal '%' in the filename. Without
    // escape_like_pattern, the '%' would behave as a wildcard and
    // either over-match or fail to match.
    insert_at(&db, "/m/percent_test/100%.flac", Some(1), Some(1), "Hundred");
    // A second literal-percent path that should NOT show up under
    // /m/percent_test (different parent).
    insert_at(
        &db,
        "/m/percent_other/200%.flac",
        Some(1),
        Some(1),
        "Two Hundred",
    );

    let children = db.list_folder_children("/m/percent_test", false).unwrap();
    assert_eq!(children.len(), 1, "only the local 100% file matches");
    match &children[0] {
        FolderTreeEntry::Track { segment, path } => {
            assert_eq!(segment, "100%.flac");
            assert_eq!(path, "/m/percent_test/100%.flac");
        }
        other => panic!("expected single track, got {:?}", other),
    }

    // And vice-versa — also test underscore handling in the parent.
    // /m/percent_test contains an '_' char that LIKE would match
    // any single character. If escape_like_pattern is missing, a
    // sibling like /m/percentXtest/foo.flac would also match.
    insert_at(&db, "/m/percentXtest/decoy.flac", Some(1), Some(1), "Decoy");
    let children = db.list_folder_children("/m/percent_test", false).unwrap();
    assert_eq!(
        children.len(),
        1,
        "underscore in parent path must be escaped"
    );
}
