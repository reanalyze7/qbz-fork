use rusqlite::Connection;

use super::db::init_schema;
use super::model::AlbumPlayMeta;
use super::queries::{query_on, record_on};

fn meta<'a>(id: &'a str, title: &'a str) -> AlbumPlayMeta<'a> {
    AlbumPlayMeta {
        album_id: id,
        title,
        artist: "Artist",
        artist_id: "7",
        artwork_url: "http://art",
        quality_tier: "hires",
        quality_label: "Hi-Res",
        year: "2024",
        source: "qobuz",
    }
}

fn mem() -> Connection {
    let c = Connection::open_in_memory().unwrap();
    init_schema(&c).unwrap();
    c
}

#[test]
fn ranks_by_play_count_desc() {
    let c = mem();
    // A: 12 plays, B: 20 plays, C: 36 plays (per-track-start counting).
    for i in 0..12 {
        record_on(&c, &meta("A", "Album A"), 100 + i);
    }
    for i in 0..20 {
        record_on(&c, &meta("B", "Album B"), 200 + i);
    }
    for i in 0..36 {
        record_on(&c, &meta("C", "Album C"), 300 + i);
    }
    let rows = query_on(&c, None);
    assert_eq!(
        rows.iter().map(|r| (r.album_id.as_str(), r.plays)).collect::<Vec<_>>(),
        vec![("C", 36), ("B", 20), ("A", 12)]
    );
    // Meta round-trips.
    assert_eq!(rows[0].title, "Album C");
    assert_eq!(rows[0].artist, "Artist");
    assert_eq!(rows[0].quality_tier, "hires");
}

#[test]
fn tie_break_prefers_more_recent_play() {
    let c = mem();
    // Both albums have 2 plays; B's last play is later -> B leads.
    record_on(&c, &meta("A", "A"), 10);
    record_on(&c, &meta("A", "A"), 11);
    record_on(&c, &meta("B", "B"), 20);
    record_on(&c, &meta("B", "B"), 21);
    let rows = query_on(&c, None);
    assert_eq!(rows.iter().map(|r| r.album_id.clone()).collect::<Vec<_>>(), vec!["B", "A"]);
}

#[test]
fn limit_caps_the_carousel() {
    let c = mem();
    for n in 0..5 {
        let id = format!("id{n}");
        // n+1 plays so ordering is deterministic (id4 highest).
        for i in 0..=n {
            record_on(&c, &meta(&id, "t"), 100 + n as i64 * 10 + i as i64);
        }
    }
    let top = query_on(&c, Some(3));
    assert_eq!(top.len(), 3);
    assert_eq!(top.iter().map(|r| r.album_id.clone()).collect::<Vec<_>>(), vec!["id4", "id3", "id2"]);
}

#[test]
fn empty_history_is_empty() {
    let c = mem();
    assert!(query_on(&c, None).is_empty());
    assert!(query_on(&c, Some(20)).is_empty());
}

#[test]
fn meta_upsert_refreshes_on_replay() {
    let c = mem();
    record_on(&c, &meta("A", "Old Title"), 1);
    let mut m2 = meta("A", "New Title");
    m2.artwork_url = "http://new-art";
    record_on(&c, &m2, 2);
    let rows = query_on(&c, None);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].title, "New Title");
    assert_eq!(rows[0].artwork_url, "http://new-art");
    assert_eq!(rows[0].plays, 2);
}
