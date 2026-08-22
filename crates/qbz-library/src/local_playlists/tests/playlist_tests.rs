//! Playlist-header CRUD tests: create/rename/flags/list/delete/migration.

use rusqlite::Connection;

use super::common::fresh_db;
use crate::local_playlists::*;

#[test]
fn create_assigns_namespaced_id_and_roundtrips() {
    let conn = fresh_db();
    let id = create(&conn, "Road Trip", Some("desc"), false).unwrap();
    assert!(is_local_playlist_id(&id), "id must carry the local: prefix");
    let p = get(&conn, &id).unwrap().unwrap();
    assert_eq!(p.name, "Road Trip");
    assert_eq!(p.description.as_deref(), Some("desc"));
    assert!(!p.offline_only);
    assert_eq!(p.track_count, 0);
}

#[test]
fn offline_only_flag_persists_and_flips() {
    let conn = fresh_db();
    let id = create(&conn, "Vault", None, true).unwrap();
    assert!(get(&conn, &id).unwrap().unwrap().offline_only);
    set_offline_only(&conn, &id, false).unwrap();
    assert!(!get(&conn, &id).unwrap().unwrap().offline_only);
}

#[test]
fn rename_and_description_update() {
    let conn = fresh_db();
    let id = create(&conn, "Old", None, false).unwrap();
    rename(&conn, &id, "New").unwrap();
    set_description(&conn, &id, Some("hello")).unwrap();
    let p = get(&conn, &id).unwrap().unwrap();
    assert_eq!(p.name, "New");
    assert_eq!(p.description.as_deref(), Some("hello"));
}

#[test]
fn favorite_and_hidden_default_false_and_flip() {
    let conn = fresh_db();
    let id = create(&conn, "Flags", None, false).unwrap();
    let p = get(&conn, &id).unwrap().unwrap();
    assert!(!p.favorite);
    assert!(!p.hidden);
    set_favorite(&conn, &id, true).unwrap();
    set_hidden(&conn, &id, true).unwrap();
    let p = get(&conn, &id).unwrap().unwrap();
    assert!(p.favorite);
    assert!(p.hidden);
    set_favorite(&conn, &id, false).unwrap();
    let p = get(&conn, &id).unwrap().unwrap();
    assert!(!p.favorite);
    assert!(p.hidden, "flags flip independently");
    // `list` carries the flags too.
    let all = list(&conn).unwrap();
    assert!(all.iter().any(|p| p.id == id && p.hidden && !p.favorite));
}

#[test]
fn init_schema_migrates_pre_b3_table() {
    // A DB created with the pre-B3 shape (no favorite/hidden columns)
    // plus an existing row: init_schema adds the columns with their
    // defaults and leaves the row readable.
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(
        r#"
        CREATE TABLE local_playlists (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            offline_only INTEGER NOT NULL DEFAULT 0,
            custom_artwork_path TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );
        INSERT INTO local_playlists (id, name, offline_only, created_at, updated_at)
        VALUES ('local:pre-b3', 'Old Row', 0, 1, 1);
        "#,
    )
    .unwrap();
    init_schema(&conn).unwrap();
    let p = get(&conn, "local:pre-b3").unwrap().unwrap();
    assert_eq!(p.name, "Old Row");
    assert!(!p.favorite);
    assert!(!p.hidden);
    // The migrated columns are writable.
    set_hidden(&conn, "local:pre-b3", true).unwrap();
    assert!(get(&conn, "local:pre-b3").unwrap().unwrap().hidden);
    // Idempotent: a second init_schema doesn't re-ALTER.
    init_schema(&conn).unwrap();
}

#[test]
fn delete_cascades_membership_rows() {
    let conn = fresh_db();
    let id = create(&conn, "Doomed", None, true).unwrap();
    add_tracks(&conn, &id, &[LocalPlaylistTrackInput::Qobuz(42)]).unwrap();
    delete(&conn, &id).unwrap();
    assert!(get(&conn, &id).unwrap().is_none());
    assert!(get_tracks(&conn, &id).unwrap().is_empty());
    // The membership table holds no orphans.
    let orphans: i64 = conn
        .query_row("SELECT COUNT(*) FROM local_playlist_tracks", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(orphans, 0);
}

#[test]
fn list_returns_all_with_counts() {
    let conn = fresh_db();
    let a = create(&conn, "A", None, false).unwrap();
    let b = create(&conn, "B", None, true).unwrap();
    add_tracks(&conn, &a, &[LocalPlaylistTrackInput::Qobuz(5)]).unwrap();
    let all = list(&conn).unwrap();
    assert_eq!(all.len(), 2);
    let pa = all.iter().find(|p| p.id == a).unwrap();
    let pb = all.iter().find(|p| p.id == b).unwrap();
    assert_eq!(pa.track_count, 1);
    assert!(!pa.offline_only);
    assert_eq!(pb.track_count, 0);
    assert!(pb.offline_only);
}
