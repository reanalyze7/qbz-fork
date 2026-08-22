use super::fresh_db;
use crate::repo::*;
use qbz_models::mixtape::{CollectionKind, CollectionPlayMode, CollectionSourceType};

#[test]
fn create_then_get_roundtrips() {
    let conn = fresh_db();
    let c = create_collection(
        &conn,
        CollectionKind::Mixtape,
        "90s Cassettes",
        Some("cassette-style"),
        CollectionSourceType::Manual,
        None,
    )
    .unwrap();
    let loaded = get_collection(&conn, &c.id).unwrap().unwrap();
    assert_eq!(loaded.name, "90s Cassettes");
    assert!(matches!(loaded.kind, CollectionKind::Mixtape));
    assert!(matches!(loaded.play_mode, CollectionPlayMode::InOrder));
    assert!(loaded.items.is_empty());
}

#[test]
fn artist_collection_stores_source_ref() {
    let conn = fresh_db();
    let c = create_collection(
        &conn,
        CollectionKind::ArtistCollection,
        "George Harrison",
        None,
        CollectionSourceType::ArtistDiscography,
        Some("qobuz-artist-123"),
    )
    .unwrap();
    assert_eq!(c.source_ref.as_deref(), Some("qobuz-artist-123"));
    assert!(matches!(c.source_type, CollectionSourceType::ArtistDiscography));
    assert!(c.last_synced_at.is_some(), "artist collection stamps last_synced_at on create");
}

#[test]
fn list_sorts_by_position_within_kind() {
    let conn = fresh_db();
    let a = create_collection(&conn, CollectionKind::Mixtape, "A", None, CollectionSourceType::Manual, None).unwrap();
    let b = create_collection(&conn, CollectionKind::Mixtape, "B", None, CollectionSourceType::Manual, None).unwrap();
    let c = create_collection(&conn, CollectionKind::Mixtape, "C", None, CollectionSourceType::Manual, None).unwrap();
    // New collections go to position=0; older ones shift. So C is first.
    let list = list_collections(&conn, Some(CollectionKind::Mixtape)).unwrap();
    assert_eq!(list[0].id, c.id);
    assert_eq!(list[1].id, b.id);
    assert_eq!(list[2].id, a.id);
}

#[test]
fn convert_kind_rejects_artist_collection() {
    let conn = fresh_db();
    let art = create_collection(
        &conn,
        CollectionKind::ArtistCollection,
        "GH",
        None,
        CollectionSourceType::ArtistDiscography,
        Some("artist-42"),
    )
    .unwrap();
    let err = set_kind(&conn, &art.id, CollectionKind::Mixtape);
    assert!(err.is_err(), "converting from artist_collection must be rejected");

    let m = create_collection(&conn, CollectionKind::Mixtape, "m", None, CollectionSourceType::Manual, None).unwrap();
    let err2 = set_kind(&conn, &m.id, CollectionKind::ArtistCollection);
    assert!(err2.is_err(), "converting to artist_collection must be rejected");
}

#[test]
fn touch_play_bumps_count_and_timestamp() {
    let conn = fresh_db();
    let c = create_collection(&conn, CollectionKind::Mixtape, "x", None, CollectionSourceType::Manual, None).unwrap();
    assert_eq!(c.play_count, 0);
    touch_play(&conn, &c.id).unwrap();
    let loaded = get_collection(&conn, &c.id).unwrap().unwrap();
    assert_eq!(loaded.play_count, 1);
    assert!(loaded.last_played_at.is_some());
}
