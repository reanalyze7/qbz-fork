//! `record_play`: append a play event and refresh the artist name cache.

use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::params;

use super::db::with_db;

/// Record a play. Called when a track starts audible playback so the
/// per-artist count converges on the user's listening reality.
#[allow(dead_code)] // wired by playback::record_recent
pub fn record_play(artist_id: u64, artist_name: &str) {
    if artist_id == 0 || artist_name.is_empty() {
        return;
    }
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    with_db(|conn| {
        if let Err(e) = conn.execute(
            "INSERT INTO play_events (artist_id, occurred_at) VALUES (?, ?)",
            params![artist_id as i64, now],
        ) {
            log::warn!("[qbz-slint] play_history insert event failed: {e}");
        }
        if let Err(e) = conn.execute(
            r#"
            INSERT INTO artist_names (artist_id, name, updated_at)
            VALUES (?, ?, ?)
            ON CONFLICT(artist_id) DO UPDATE SET
                name = excluded.name,
                updated_at = excluded.updated_at
            "#,
            params![artist_id as i64, artist_name, now],
        ) {
            log::warn!("[qbz-slint] play_history upsert name failed: {e}");
        }
        Some(())
    });
}
