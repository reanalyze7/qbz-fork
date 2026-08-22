//! Shared fixtures for the `folder_tree` test suite.
//!
//! Fixture layout (paths only — metadata is enough to round-trip
//! through `LocalTrack`):
//!
//! ```text
//! /m/A/album1/t1.flac           (user)
//! /m/A/album1/t2.flac           (user)
//! /m/A/album1/Disc 1/t3.flac    (user, in subfolder)
//! /m/A/album2/t4.flac           (user)
//! /m/B/album3/t5.flac           (user)
//! /m/A/album1/qcache.flac       (qobuz_download — must be filtered)
//! /m/percent_test/100%.flac     (user, special chars)
//! ```

use tempfile::TempDir;

use crate::database::LibraryDatabase;
use crate::LocalTrack;

pub(super) fn fresh_db() -> (TempDir, LibraryDatabase) {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("library.db");
    let db = LibraryDatabase::open(&path).unwrap();
    (tmp, db)
}

pub(super) fn insert_at(
    db: &LibraryDatabase,
    file_path: &str,
    disc: Option<u32>,
    track_no: Option<u32>,
    title: &str,
) {
    // NB: `LibraryDatabase::insert_track` stamps `source` itself
    // (always 'user' unless the path matches a downloaded_purchases
    // row), so we never set track.source here. To insert with a
    // different source value (e.g. 'qobuz_download'), use
    // `insert_qobuz_download_at` below.
    let mut t = LocalTrack::default();
    t.file_path = file_path.to_string();
    t.title = title.to_string();
    t.album = "Test Album".to_string();
    t.album_artist = Some("Test Artist".to_string());
    t.artist = "Test Artist".to_string();
    t.album_group_key = file_path
        .rsplit_once('/')
        .map(|(parent, _)| parent.to_string())
        .unwrap_or_default();
    t.album_group_title = "Test Album".to_string();
    t.disc_number = disc;
    t.track_number = track_no;
    db.insert_track(&t).unwrap();
}

/// Insert a row directly with `source = 'qobuz_download'`.
/// `insert_track` overrides the source field so we go through raw
/// SQL to model the offline-cache code path that DOES write that
/// value.
pub(super) fn insert_qobuz_download_at(db: &LibraryDatabase, file_path: &str, title: &str) {
    db.with_connection(|conn| {
        conn.execute(
            "INSERT INTO local_tracks \
             (file_path, title, artist, album, album_artist, \
              track_number, disc_number, year, genre, catalog_number, \
              duration_secs, format, bit_depth, sample_rate, channels, \
              file_size_bytes, cue_file_path, cue_start_secs, cue_end_secs, \
              artwork_path, last_modified, indexed_at, album_group_key, \
              album_group_title, source, is_network_mount) \
             VALUES (?1, ?2, 'X', 'X', 'X', \
                     1, 1, NULL, NULL, NULL, \
                     0, 'FLAC', NULL, 44100.0, 2, \
                     0, NULL, NULL, NULL, \
                     NULL, 0, 0, 'qcache', \
                     'qcache', 'qobuz_download', 0)",
            rusqlite::params![file_path, title],
        )
        .unwrap();
    });
}

pub(super) fn seed_standard_fixture(db: &LibraryDatabase) {
    // Standard layout — 5 user tracks under /m/A and /m/B.
    insert_at(db, "/m/A/album1/t1.flac", Some(1), Some(1), "Alpha");
    insert_at(db, "/m/A/album1/t2.flac", Some(1), Some(2), "Beta");
    insert_at(db, "/m/A/album1/Disc 1/t3.flac", Some(1), Some(1), "Gamma");
    insert_at(db, "/m/A/album2/t4.flac", Some(1), Some(1), "Delta");
    insert_at(db, "/m/B/album3/t5.flac", Some(1), Some(1), "Epsilon");

    // One Qobuz download in the same album — must be filtered out
    // by both list_folder_children and list_folder_tracks.
    insert_qobuz_download_at(db, "/m/A/album1/qcache.flac", "QobuzCache");
}
