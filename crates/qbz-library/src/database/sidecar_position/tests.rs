use super::super::LibraryDatabase;
use crate::LocalTrack;
use rusqlite::params;
use tempfile::TempDir;

fn fresh_db() -> (TempDir, LibraryDatabase) {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("library.db");
    let db = LibraryDatabase::open(&path).unwrap();
    (tmp, db)
}

/// Real `local_tracks` rows for the FK on `playlist_local_tracks`.
/// Returns the library row ids in insertion order.
fn seed_local_tracks(db: &LibraryDatabase, count: usize) -> Vec<i64> {
    (0..count)
        .map(|i| {
            let mut t = LocalTrack::default();
            t.file_path = format!("/t/track{i}.flac");
            t.title = format!("T{i}");
            t.artist = "A".into();
            t.album = "B".into();
            db.insert_track(&t).unwrap()
        })
        .collect()
}

fn local_positions(db: &LibraryDatabase, pid: u64) -> Vec<(i64, i32)> {
    let mut stmt = db
        .conn
        .prepare(
            "SELECT local_track_id, position FROM playlist_local_tracks
             WHERE qobuz_playlist_id = ?1 ORDER BY local_track_id ASC",
        )
        .unwrap();
    stmt.query_map(params![pid as i64], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
}

#[test]
fn next_position_empty_sidecar_appends_after_qobuz_block() {
    let (_tmp, db) = fresh_db();
    assert_eq!(db.next_playlist_sidecar_position(7, 50).unwrap(), 50);
    assert_eq!(db.next_playlist_sidecar_position(7, 0).unwrap(), 0);
}

#[test]
fn next_position_dense_positions_match_count_formula() {
    let (_tmp, db) = fresh_db();
    let ids = seed_local_tracks(&db, 2);
    db.add_local_track_to_playlist(7, ids[0], 50).unwrap();
    db.add_local_track_to_playlist(7, ids[1], 51).unwrap();
    // count-based 50+2 == max+1 == 52.
    assert_eq!(db.next_playlist_sidecar_position(7, 50).unwrap(), 52);
}

#[test]
fn next_position_gapped_positions_clear_the_stored_max() {
    let (_tmp, db) = fresh_db();
    // T3 regression: positions keep gaps after removals; the count
    // formula alone would re-issue 52 while 80 is still stored.
    let ids = seed_local_tracks(&db, 2);
    db.add_local_track_to_playlist(7, ids[0], 50).unwrap();
    db.add_local_track_to_playlist(7, ids[1], 80).unwrap();
    assert_eq!(db.next_playlist_sidecar_position(7, 50).unwrap(), 81);
}

#[test]
fn next_position_legacy_low_positions_fall_back_to_counts() {
    let (_tmp, db) = fresh_db();
    // Legacy 0-based rows: max+1 == 2, but the merged list is 52 long.
    let ids = seed_local_tracks(&db, 2);
    db.add_local_track_to_playlist(7, ids[0], 0).unwrap();
    db.add_local_track_to_playlist(7, ids[1], 1).unwrap();
    assert_eq!(db.next_playlist_sidecar_position(7, 50).unwrap(), 52);
}

#[test]
fn next_position_scoped_per_playlist() {
    let (_tmp, db) = fresh_db();
    let ids = seed_local_tracks(&db, 1);
    db.add_local_track_to_playlist(7, ids[0], 99).unwrap();
    assert_eq!(db.next_playlist_sidecar_position(8, 10).unwrap(), 10);
}

#[test]
fn heal_without_collisions_is_a_noop() {
    let (_tmp, db) = fresh_db();
    let ids = seed_local_tracks(&db, 3);
    db.add_local_track_to_playlist(7, ids[0], 0).unwrap();
    db.add_local_track_to_playlist(7, ids[1], 5).unwrap();
    db.add_local_track_to_playlist(7, ids[2], 9).unwrap();
    let healed = db.heal_playlist_sidecar_positions(7, 50).unwrap();
    assert!(healed.is_empty(), "drift is normal (E7): {healed:?}");
    assert_eq!(
        local_positions(&db, 7),
        vec![(ids[0], 0), (ids[1], 5), (ids[2], 9)]
    );
}

#[test]
fn heal_within_table_collision_moves_the_later_claimant() {
    let (_tmp, db) = fresh_db();
    // Two legacy 0-based batches: 0,1 then 0 again (E1).
    let ids = seed_local_tracks(&db, 3);
    db.add_local_track_to_playlist(7, ids[0], 0).unwrap();
    db.add_local_track_to_playlist(7, ids[1], 1).unwrap();
    db.add_local_track_to_playlist(7, ids[2], 0).unwrap();
    let healed = db.heal_playlist_sidecar_positions(7, 10).unwrap();
    assert_eq!(healed.len(), 1, "{healed:?}");
    // First claimant (rowid order on the added_at tie) keeps slot 0;
    // the later one moves to the append region: max(10+3, 1+1) = 13.
    assert_eq!(
        local_positions(&db, 7),
        vec![(ids[0], 0), (ids[1], 1), (ids[2], 13)]
    );
}

#[test]
fn heal_is_idempotent() {
    let (_tmp, db) = fresh_db();
    let ids = seed_local_tracks(&db, 2);
    db.add_local_track_to_playlist(7, ids[0], 0).unwrap();
    db.add_local_track_to_playlist(7, ids[1], 0).unwrap();
    assert!(!db.heal_playlist_sidecar_positions(7, 5).unwrap().is_empty());
    assert!(db.heal_playlist_sidecar_positions(7, 5).unwrap().is_empty());
}
