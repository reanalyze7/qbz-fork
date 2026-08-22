//! Tests for `db/album.rs`'s album-scoped bulk operations.

use crate::db::OfflineCacheDb;
use crate::types::{OfflineCacheStatus, TrackCacheInfo};
use tempfile::TempDir;

fn fresh_db() -> (TempDir, OfflineCacheDb) {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("idx.db");
    let db = OfflineCacheDb::new(&path).unwrap();
    (tmp, db)
}

fn sample_track(id: u64, album_id: Option<&str>) -> TrackCacheInfo {
    TrackCacheInfo {
        track_id: id,
        title: format!("t{}", id),
        artist: "A".into(),
        album: Some("Alb".into()),
        album_id: album_id.map(String::from),
        duration_secs: 100,
        quality: "lossless".into(),
        bit_depth: Some(16),
        sample_rate: Some(44100.0),
    }
}

#[test]
fn delete_album_tracks_returns_deleted_ids_and_freed_bytes() {
    let (_tmp, db) = fresh_db();
    db.insert_track(&sample_track(1, Some("alb1")), "/p/1").unwrap();
    db.insert_track(&sample_track(2, Some("alb1")), "/p/2").unwrap();
    db.insert_track(&sample_track(3, Some("alb2")), "/p/3").unwrap();
    db.mark_complete(1, 1000).unwrap();
    db.mark_complete(2, 2000).unwrap();
    db.mark_complete(3, 9999).unwrap();

    let (ids, bytes) = db.delete_album_tracks("alb1").unwrap();
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&1) && ids.contains(&2));
    assert_eq!(bytes, 3000);

    // alb2 untouched
    let remaining = db.get_all_tracks().unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].track_id, 3);
}

#[test]
fn get_album_tracks_returns_only_matching_album() {
    let (_tmp, db) = fresh_db();
    db.insert_track(&sample_track(1, Some("alb1")), "/p/1").unwrap();
    db.insert_track(&sample_track(2, Some("alb2")), "/p/2").unwrap();

    let alb1 = db.get_album_tracks("alb1").unwrap();
    assert_eq!(alb1.len(), 1);
    assert_eq!(alb1[0].track_id, 1);
}

#[test]
fn reset_track_for_redownload_clears_progress_and_error() {
    let (_tmp, db) = fresh_db();
    db.insert_track(&sample_track(1, Some("alb1")), "/p/1").unwrap();
    db.update_status(1, OfflineCacheStatus::Failed, Some("boom")).unwrap();

    db.reset_track_for_redownload(1).unwrap();

    let track = db.get_track(1).unwrap().unwrap();
    assert!(matches!(track.status, OfflineCacheStatus::Queued));
    assert_eq!(track.progress_percent, 0);
    assert!(track.error_message.is_none());
}
