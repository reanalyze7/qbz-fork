//! Tests for the network-folder exclusion toggle across every
//! tree-mode listing primitive (`list_folder_children`,
//! `list_folder_tracks`, `list_folder_tracks_recursive`). See
//! `network_count_tests` for the `count_folder_tracks_recursive`
//! companion tests.

use crate::FolderTreeEntry;

use super::common::{fresh_db, insert_at};

/// Mirrors the offline / "exclude network folders" toggle: tracks
/// living under a `library_folders` row marked `is_network = 1`
/// must be filtered out of every tree-mode listing primitive when
/// `exclude_network_folders = true`, and present when `false`.
/// Matches the predicate used by `get_albums_with_full_filter`
/// and `v2_library_search` so flat mode and tree mode see the same
/// source of truth.
#[test]
fn list_folder_primitives_honor_network_exclude() {
    let (_tmp, db) = fresh_db();

    // Register two scan roots: one local, one flagged as network.
    db.add_folder_with_network_info("/m/local", false, None)
        .unwrap();
    db.add_folder_with_network_info("/m/net", true, Some("nfs"))
        .unwrap();

    // Seed user tracks under each root. The folder structure is
    // similar enough that the only thing distinguishing them is the
    // network-mount flag on the parent library_folders row.
    insert_at(&db, "/m/local/album/local1.flac", Some(1), Some(1), "L1");
    insert_at(&db, "/m/local/album/local2.flac", Some(1), Some(2), "L2");
    insert_at(&db, "/m/net/album/net1.flac", Some(1), Some(1), "N1");
    insert_at(&db, "/m/net/album/sub/net2.flac", Some(1), Some(1), "N2");

    // --- list_folder_children -----------------------------------
    // Without filter: both roots appear under '/m'.
    let all_children = db.list_folder_children("/m", false).unwrap();
    let segments: Vec<_> = all_children
        .iter()
        .filter_map(|e| match e {
            FolderTreeEntry::Folder { segment, .. } => Some(segment.as_str()),
            _ => None,
        })
        .collect();
    assert!(segments.contains(&"local"));
    assert!(segments.contains(&"net"));

    // With filter: the network root collapses out (no descendant
    // tracks survive the EXISTS predicate, so it stops aggregating).
    let filtered = db.list_folder_children("/m", true).unwrap();
    let segments: Vec<_> = filtered
        .iter()
        .filter_map(|e| match e {
            FolderTreeEntry::Folder { segment, .. } => Some(segment.as_str()),
            _ => None,
        })
        .collect();
    assert!(segments.contains(&"local"));
    assert!(
        !segments.contains(&"net"),
        "network folder leaked into tree rail when exclude=true"
    );

    // --- list_folder_tracks (direct children) ------------------
    let direct_all = db.list_folder_tracks("/m/net/album", false).unwrap();
    assert_eq!(direct_all.len(), 1, "net1.flac must appear when exclude=false");

    let direct_filtered = db.list_folder_tracks("/m/net/album", true).unwrap();
    assert!(
        direct_filtered.is_empty(),
        "network track leaked into direct-children listing when exclude=true"
    );

    // Local folder is unaffected by the toggle.
    let local_filtered = db.list_folder_tracks("/m/local/album", true).unwrap();
    assert_eq!(local_filtered.len(), 2);

    // --- list_folder_tracks_recursive --------------------------
    let recursive_all = db.list_folder_tracks_recursive("/m/net", false).unwrap();
    assert_eq!(recursive_all.len(), 2, "both net tracks visible when exclude=false");

    let recursive_filtered = db
        .list_folder_tracks_recursive("/m/net", true)
        .unwrap();
    assert!(
        recursive_filtered.is_empty(),
        "network tracks leaked into recursive listing when exclude=true"
    );

    // Recursive listing on a non-network root still returns its
    // tracks even when exclude=true.
    let recursive_local = db
        .list_folder_tracks_recursive("/m/local", true)
        .unwrap();
    assert_eq!(recursive_local.len(), 2);
}
