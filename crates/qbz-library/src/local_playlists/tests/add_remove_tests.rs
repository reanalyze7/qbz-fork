//! Membership tests: add/dedupe/remove.

use super::common::fresh_db;
use crate::local_playlists::*;

#[test]
fn add_tracks_appends_positions_across_sources() {
    let conn = fresh_db();
    let id = create(&conn, "Mixed", None, false).unwrap();
    let n = add_tracks(
        &conn,
        &id,
        &[
            LocalPlaylistTrackInput::Qobuz(111),
            LocalPlaylistTrackInput::Local("/music/a.flac".into()),
        ],
    )
    .unwrap();
    assert_eq!(n, 2);
    // Second batch continues the position sequence.
    let n2 = add_tracks(&conn, &id, &[LocalPlaylistTrackInput::Qobuz(222)]).unwrap();
    assert_eq!(n2, 1);

    let rows = get_tracks(&conn, &id).unwrap();
    assert_eq!(rows.len(), 3);
    assert_eq!(
        rows.iter().map(|r| r.position).collect::<Vec<_>>(),
        vec![0, 1, 2]
    );
    assert_eq!(rows[0].qobuz_track_id, Some(111));
    assert_eq!(rows[1].local_path.as_deref(), Some("/music/a.flac"));
    assert_eq!(rows[2].qobuz_track_id, Some(222));

    let p = get(&conn, &id).unwrap().unwrap();
    assert_eq!(p.track_count, 3);
    assert_eq!(p.qobuz_count, 2);
    assert_eq!(p.local_count, 1);
}

#[test]
fn add_tracks_skips_exact_duplicates() {
    let conn = fresh_db();
    let id = create(&conn, "Dedupe", None, false).unwrap();
    add_tracks(&conn, &id, &[LocalPlaylistTrackInput::Qobuz(7)]).unwrap();
    let n = add_tracks(
        &conn,
        &id,
        &[
            LocalPlaylistTrackInput::Qobuz(7),
            LocalPlaylistTrackInput::Local("/x.flac".into()),
        ],
    )
    .unwrap();
    assert_eq!(n, 1, "duplicate qobuz id skipped, new local row inserted");
    assert_eq!(get_tracks(&conn, &id).unwrap().len(), 2);
}

#[test]
fn remove_track_compacts_positions() {
    let conn = fresh_db();
    let id = create(&conn, "Compact", None, false).unwrap();
    add_tracks(
        &conn,
        &id,
        &[
            LocalPlaylistTrackInput::Qobuz(1),
            LocalPlaylistTrackInput::Qobuz(2),
            LocalPlaylistTrackInput::Qobuz(3),
        ],
    )
    .unwrap();
    remove_track(&conn, &id, 1).unwrap();
    let rows = get_tracks(&conn, &id).unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].qobuz_track_id, Some(1));
    assert_eq!(rows[0].position, 0);
    assert_eq!(rows[1].qobuz_track_id, Some(3));
    assert_eq!(rows[1].position, 1);
}
