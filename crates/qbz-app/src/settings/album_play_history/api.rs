use std::time::{SystemTime, UNIX_EPOCH};

use super::db::with_db;
use super::model::{AlbumPlayMeta, AlbumPlayRow};
use super::queries::{query_on, record_on};

/// Record a play. Called from `playback::record_recent` when a track starts
/// audible playback. No-op when the album id is empty (some local
/// sources carry none — same guard as the recently-played rail).
#[allow(dead_code)] // wired by playback::record_recent
pub fn record_album_play(m: AlbumPlayMeta) {
    if m.album_id.is_empty() {
        return;
    }
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    with_db(|conn| {
        record_on(conn, &m, now);
        Some(())
    });
}

/// The top `limit` most-played albums (the carousel).
#[allow(dead_code)] // wired by home/foryou
pub fn top_albums(limit: u32) -> Vec<AlbumPlayRow> {
    with_db(|conn| Some(query_on(conn, Some(limit)))).unwrap_or_default()
}

/// Every played album, ranked (the "View all" page).
#[allow(dead_code)] // wired by the View-all loader
pub fn all_albums() -> Vec<AlbumPlayRow> {
    with_db(|conn| Some(query_on(conn, None))).unwrap_or_default()
}
