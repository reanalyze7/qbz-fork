//! Shared test fixtures for the `albums::metadata_grouped` test suite.

use tempfile::TempDir;

use crate::database::LibraryDatabase;
use crate::LocalTrack;

pub(super) fn fresh_db() -> (TempDir, LibraryDatabase) {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("library.db");
    let db = LibraryDatabase::open(&path).unwrap();
    (tmp, db)
}

pub(super) fn insert_track_for_test(
    db: &LibraryDatabase,
    file_path: &str,
    album: Option<&str>,
    album_artist: Option<&str>,
    artist: &str,
    album_group_key: &str,
) {
    let mut t = LocalTrack::default();
    t.file_path = file_path.to_string();
    t.title = format!("Track at {}", file_path);
    t.album = album.unwrap_or("").to_string();
    t.album_artist = album_artist.map(String::from);
    t.artist = artist.to_string();
    t.album_group_key = album_group_key.to_string();
    t.album_group_title = album.unwrap_or("").to_string();
    db.insert_track(&t).unwrap();
}

/// Like `insert_track_for_test`, but with control over
/// `album_group_title` (the scan-time snapshot — folder name when the
/// tag was missing) and `year`, for the #447/#507 regressions.
#[allow(clippy::too_many_arguments)]
pub(super) fn insert_full_track_for_test(
    db: &LibraryDatabase,
    file_path: &str,
    album: &str,
    album_artist: Option<&str>,
    artist: &str,
    album_group_key: &str,
    album_group_title: &str,
    year: Option<u32>,
) {
    let mut t = LocalTrack::default();
    t.file_path = file_path.to_string();
    t.title = format!("Track at {}", file_path);
    t.album = album.to_string();
    t.album_artist = album_artist.map(String::from);
    t.artist = artist.to_string();
    t.album_group_key = album_group_key.to_string();
    t.album_group_title = album_group_title.to_string();
    t.year = year;
    db.insert_track(&t).unwrap();
}
