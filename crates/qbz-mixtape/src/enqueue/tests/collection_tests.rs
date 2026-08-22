use super::item;
use super::mock_resolver::MockResolver;
use crate::enqueue::resolve_collection_tracks;
use qbz_models::mixtape::{AlbumSource, CollectionPlayMode, ItemType};

#[tokio::test]
async fn resolver_stamps_hint_and_flattens_in_order() {
    let items = vec![
        item(0, ItemType::Album, AlbumSource::Qobuz, "a-1", 3),
        item(1, ItemType::Track, AlbumSource::Qobuz, "t-99", 1),
        item(2, ItemType::Album, AlbumSource::Local, "al-local-xyz", 2),
    ];
    let tracks =
        resolve_collection_tracks(items, CollectionPlayMode::InOrder, &MockResolver).await;
    assert_eq!(tracks.len(), 6);
    assert_eq!(tracks[0].source_item_id_hint.as_deref(), Some("a-1"));
    assert_eq!(tracks[2].source_item_id_hint.as_deref(), Some("a-1"));
    assert_eq!(tracks[3].source_item_id_hint.as_deref(), Some("t-99"));
    assert_eq!(tracks[4].source_item_id_hint.as_deref(), Some("al-local-xyz"));
}

#[tokio::test]
async fn album_shuffle_changes_order_but_each_album_stays_together() {
    let items = vec![
        item(0, ItemType::Album, AlbumSource::Qobuz, "a-1", 3),
        item(1, ItemType::Album, AlbumSource::Qobuz, "a-2", 3),
        item(2, ItemType::Album, AlbumSource::Qobuz, "a-3", 3),
    ];
    let tracks =
        resolve_collection_tracks(items, CollectionPlayMode::AlbumShuffle, &MockResolver)
            .await;
    assert_eq!(tracks.len(), 9);
    // Each album's tracks must be contiguous (no interleaving).
    let mut i = 0;
    let mut seen = std::collections::HashSet::new();
    while i < tracks.len() {
        let h = tracks[i].source_item_id_hint.clone().unwrap();
        assert!(
            !seen.contains(&h),
            "album {} must not reappear after a gap",
            h
        );
        seen.insert(h.clone());
        while i < tracks.len()
            && tracks[i].source_item_id_hint.as_deref() == Some(&h)
        {
            i += 1;
        }
    }
    assert_eq!(seen.len(), 3, "all 3 albums must be represented");
}
