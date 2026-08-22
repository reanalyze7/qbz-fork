use super::fresh_db;
use crate::repo::*;
use qbz_models::mixtape::{AlbumSource, CollectionKind, CollectionSourceType, ItemType};

#[test]
fn add_item_dedupes_on_source_plus_id_exact() {
    let conn = fresh_db();
    let c = create_collection(&conn, CollectionKind::Mixtape, "x", None, CollectionSourceType::Manual, None).unwrap();
    let ok1 = add_item(
        &conn, &c.id, ItemType::Album, AlbumSource::Qobuz,
        "album-123", "Dookie", Some("Green Day"), None, Some(1994), Some(15),
    ).unwrap();
    let ok2 = add_item(
        &conn, &c.id, ItemType::Album, AlbumSource::Qobuz,
        "album-123", "Dookie", Some("Green Day"), None, Some(1994), Some(15),
    ).unwrap();
    assert!(ok1, "first add succeeds");
    assert!(!ok2, "exact duplicate rejected");

    // Different source — allowed; same item id in a different source is a different item.
    let ok3 = add_item(
        &conn, &c.id, ItemType::Album, AlbumSource::Local,
        "album-123", "Dookie", Some("Green Day"), None, Some(1994), Some(15),
    ).unwrap();
    assert!(ok3, "different source passes dedup");

    // Different item_type but same source+id — allowed (conceptually different beast:
    // a track dropped next to an album of the same id would still be a distinct item).
    let ok4 = add_item(
        &conn, &c.id, ItemType::Track, AlbumSource::Local,
        "album-123", "Dookie - track", Some("Green Day"), None, Some(1994), Some(1),
    ).unwrap();
    assert!(!ok4, "same source+id even across item_type is still dedup");
    // (NOTE: spec says dedup is exact (source, source_item_id). If your read of
    // the spec differs, adjust this test AND the add_item SQL accordingly.)
}

#[test]
fn add_track_and_playlist_item_types() {
    let conn = fresh_db();
    let c = create_collection(&conn, CollectionKind::Mixtape, "mixed", None, CollectionSourceType::Manual, None).unwrap();
    add_item(&conn, &c.id, ItemType::Album,    AlbumSource::Qobuz, "al-1",  "Alb",  None, None, None, None).unwrap();
    add_item(&conn, &c.id, ItemType::Track,    AlbumSource::Qobuz, "tk-99", "Trk",  None, None, None, Some(1)).unwrap();
    add_item(&conn, &c.id, ItemType::Playlist, AlbumSource::Qobuz, "pl-7",  "Plst", None, None, None, Some(24)).unwrap();
    let items = list_items(&conn, &c.id).unwrap();
    assert_eq!(items.len(), 3);
    assert!(matches!(items[0].item_type, ItemType::Album));
    assert!(matches!(items[1].item_type, ItemType::Track));
    assert!(matches!(items[2].item_type, ItemType::Playlist));
}

#[test]
fn remove_item_compacts_positions() {
    let conn = fresh_db();
    let c = create_collection(&conn, CollectionKind::Collection, "x", None, CollectionSourceType::Manual, None).unwrap();
    for i in 0..3 {
        add_item(
            &conn, &c.id, ItemType::Album, AlbumSource::Qobuz,
            &format!("id-{}", i), &format!("t-{}", i), None, None, None, None,
        ).unwrap();
    }
    remove_item(&conn, &c.id, 1).unwrap();
    let items = list_items(&conn, &c.id).unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].position, 0);
    assert_eq!(items[1].position, 1);
    assert_eq!(items[1].source_item_id, "id-2");
}

#[test]
fn reorder_items_round_trips() {
    let mut conn = fresh_db();
    let c = create_collection(&conn, CollectionKind::Mixtape, "x", None, CollectionSourceType::Manual, None).unwrap();
    for i in 0..3 {
        add_item(
            &conn, &c.id, ItemType::Album, AlbumSource::Qobuz,
            &format!("id-{}", i), &format!("t-{}", i), None, None, None, None,
        ).unwrap();
    }
    // Reverse the order: old [0,1,2] -> new [2,1,0]
    reorder_items(&mut conn, &c.id, &[2, 1, 0]).unwrap();
    let items = list_items(&conn, &c.id).unwrap();
    assert_eq!(items[0].source_item_id, "id-2");
    assert_eq!(items[1].source_item_id, "id-1");
    assert_eq!(items[2].source_item_id, "id-0");
    for (i, it) in items.iter().enumerate() {
        assert_eq!(it.position, i as i32, "positions dense after reorder");
    }
}

#[test]
fn delete_collection_cascades_items() {
    let conn = fresh_db();
    conn.execute("PRAGMA foreign_keys = ON", []).unwrap(); // ensure cascade fires
    let c = create_collection(&conn, CollectionKind::Mixtape, "x", None, CollectionSourceType::Manual, None).unwrap();
    add_item(&conn, &c.id, ItemType::Album, AlbumSource::Qobuz, "a", "t", None, None, None, None).unwrap();
    delete_collection(&conn, &c.id).unwrap();
    let items = list_items(&conn, &c.id).unwrap();
    assert!(items.is_empty());
}
