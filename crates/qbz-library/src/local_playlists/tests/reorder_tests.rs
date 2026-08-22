//! Membership reorder tests, plus the `qobuz_order`/`seeded_playlist`
//! helpers they share.

use rusqlite::Connection;

use super::common::fresh_db;
use crate::local_playlists::*;

/// The repo positions, in row order, with the qobuz id per row — the
/// shape every reorder assertion checks.
fn qobuz_order(conn: &Connection, id: &str) -> Vec<(i32, u64)> {
    get_tracks(conn, id)
        .unwrap()
        .iter()
        .map(|r| (r.position, r.qobuz_track_id.unwrap()))
        .collect()
}

fn seeded_playlist(conn: &Connection, ids: &[u64]) -> String {
    let id = create(conn, "Reorder", None, false).unwrap();
    let entries: Vec<LocalPlaylistTrackInput> = ids
        .iter()
        .map(|&tid| LocalPlaylistTrackInput::Qobuz(tid))
        .collect();
    add_tracks(conn, &id, &entries).unwrap();
    id
}

#[test]
fn reorder_moves_down_with_compaction() {
    let conn = fresh_db();
    let id = seeded_playlist(&conn, &[1, 2, 3, 4]);
    // Move the first row to slot 2: [1,2,3,4] -> [2,3,1,4].
    reorder(&conn, &id, 0, 2).unwrap();
    assert_eq!(
        qobuz_order(&conn, &id),
        vec![(0, 2), (1, 3), (2, 1), (3, 4)]
    );
}

#[test]
fn reorder_moves_up_with_compaction() {
    let conn = fresh_db();
    let id = seeded_playlist(&conn, &[1, 2, 3, 4]);
    // Move the last row to slot 1: [1,2,3,4] -> [1,4,2,3].
    reorder(&conn, &id, 3, 1).unwrap();
    assert_eq!(
        qobuz_order(&conn, &id),
        vec![(0, 1), (1, 4), (2, 2), (3, 3)]
    );
}

#[test]
fn reorder_adjacent_swap() {
    let conn = fresh_db();
    let id = seeded_playlist(&conn, &[1, 2, 3]);
    reorder(&conn, &id, 1, 2).unwrap();
    assert_eq!(qobuz_order(&conn, &id), vec![(0, 1), (1, 3), (2, 2)]);
    reorder(&conn, &id, 2, 1).unwrap();
    assert_eq!(qobuz_order(&conn, &id), vec![(0, 1), (1, 2), (2, 3)]);
}

#[test]
fn reorder_noop_on_same_or_missing_positions() {
    let conn = fresh_db();
    let id = seeded_playlist(&conn, &[1, 2, 3]);
    let before = qobuz_order(&conn, &id);
    reorder(&conn, &id, 1, 1).unwrap(); // same slot
    reorder(&conn, &id, 7, 0).unwrap(); // from doesn't exist
    reorder(&conn, &id, 0, 7).unwrap(); // to doesn't exist
    reorder(&conn, &id, 0, -1).unwrap(); // negative target
    assert_eq!(qobuz_order(&conn, &id), before);
}

#[test]
fn reorder_scoped_to_its_playlist() {
    let conn = fresh_db();
    let a = seeded_playlist(&conn, &[1, 2, 3]);
    let b = seeded_playlist(&conn, &[10, 20, 30]);
    reorder(&conn, &a, 0, 2).unwrap();
    assert_eq!(qobuz_order(&conn, &a), vec![(0, 2), (1, 3), (2, 1)]);
    // The sibling playlist's rows are untouched.
    assert_eq!(qobuz_order(&conn, &b), vec![(0, 10), (1, 20), (2, 30)]);
}
