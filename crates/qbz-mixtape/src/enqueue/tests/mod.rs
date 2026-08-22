use super::*;
use qbz_models::mixtape::{AlbumSource, ItemType, MixtapeCollectionItem};

mod mock_resolver;
mod boundary_tests;
mod collection_tests;

pub(super) fn item(
    idx: i32,
    kind: ItemType,
    src: AlbumSource,
    id: &str,
    tracks: i32,
) -> MixtapeCollectionItem {
    MixtapeCollectionItem {
        collection_id: "c".into(),
        position: idx,
        item_type: kind,
        source: src,
        source_item_id: id.into(),
        title: format!("title-{}", idx),
        subtitle: None,
        artwork_url: None,
        year: None,
        track_count: Some(tracks),
        added_at: 0,
    }
}

#[test]
fn resolve_local_item_playlist_is_unsupported() {
    // The (Playlist, Local) hard error is a load-bearing contract (spec §5.4/§10);
    // lock it so the later Slint enqueue slice cannot silently drop it.
    let db = qbz_library::LibraryDatabase::open(std::path::Path::new(":memory:")).unwrap();
    let it = item(0, ItemType::Playlist, AlbumSource::Local, "whatever", 0);
    let err = resolve_local_item(&db, &it).unwrap_err();
    assert_eq!(err, "local playlists not supported in this release");
}

#[test]
fn resolve_local_item_track_rejects_non_numeric_id() {
    let db = qbz_library::LibraryDatabase::open(std::path::Path::new(":memory:")).unwrap();
    let it = item(0, ItemType::Track, AlbumSource::Local, "not-a-number", 0);
    let err = resolve_local_item(&db, &it).unwrap_err();
    assert_eq!(err, "invalid local track id: not-a-number");
}
