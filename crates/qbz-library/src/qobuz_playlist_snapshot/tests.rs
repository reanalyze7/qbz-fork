use super::*;
use rusqlite::Connection;

fn conn() -> Connection {
    let c = Connection::open_in_memory().unwrap();
    init_schema(&c).unwrap();
    c
}

fn name_entry(id: u64, name: &str, count: u32) -> SnapshotNameEntry {
    SnapshotNameEntry {
        qobuz_playlist_id: id,
        name: name.to_string(),
        owner: Some("me".to_string()),
        track_count: Some(count),
    }
}

#[test]
fn roundtrip_header_and_tracks() {
    let c = conn();
    upsert_names(&c, &[name_entry(42, "Road Trip", 3)]).unwrap();
    let wrote = replace_tracks(&c, 42, "Road Trip", Some("me"), &[30, 10, 20]).unwrap();
    assert!(wrote);

    let h = get_header(&c, 42).unwrap().unwrap();
    assert_eq!(h.name, "Road Trip");
    assert_eq!(h.owner.as_deref(), Some("me"));
    assert_eq!(h.track_count, Some(3));
    assert!(h.snapped_at > 0);

    // Snapshot order preserved, not sorted by id.
    assert_eq!(track_ids(&c, 42).unwrap(), vec![30, 10, 20]);
    let all = all_track_ids(&c).unwrap();
    assert_eq!(all.get(&42).unwrap(), &vec![30, 10, 20]);
}

#[test]
fn replace_is_full_replace() {
    let c = conn();
    upsert_names(&c, &[name_entry(7, "Mix", 3)]).unwrap();
    replace_tracks(&c, 7, "Mix", None, &[1, 2, 3]).unwrap();
    replace_tracks(&c, 7, "Mix renamed", None, &[9]).unwrap();

    assert_eq!(track_ids(&c, 7).unwrap(), vec![9]);
    let h = get_header(&c, 7).unwrap().unwrap();
    assert_eq!(h.name, "Mix renamed");
    assert_eq!(h.track_count, Some(1));
    // No leftover rows from the first write.
    let total: i64 = c
        .query_row(
            "SELECT COUNT(*) FROM qobuz_playlist_snapshot_tracks",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(total, 1);
}

#[test]
fn names_only_rows_without_tracks() {
    let c = conn();
    upsert_names(&c, &[name_entry(1, "A", 10), name_entry(2, "B", 0)]).unwrap();

    assert_eq!(all_headers(&c).unwrap().len(), 2);
    assert!(track_ids(&c, 1).unwrap().is_empty());
    assert!(all_track_ids(&c).unwrap().is_empty());

    // Re-upserting updates the name in place without creating track rows.
    upsert_names(&c, &[name_entry(1, "A renamed", 11)]).unwrap();
    let h = get_header(&c, 1).unwrap().unwrap();
    assert_eq!(h.name, "A renamed");
    assert_eq!(h.track_count, Some(11));
    assert!(track_ids(&c, 1).unwrap().is_empty());
}

#[test]
fn replace_refuses_unknown_playlist() {
    let c = conn();
    // No names row -> the detail producer writes NOTHING (a merely
    // viewed public playlist must not land in the snapshot).
    let wrote = replace_tracks(&c, 99, "Someone's list", None, &[1, 2]).unwrap();
    assert!(!wrote);
    assert!(get_header(&c, 99).unwrap().is_none());
    assert!(track_ids(&c, 99).unwrap().is_empty());
}
