use rusqlite::{Connection, Result};

pub(super) fn now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

/// Create the snapshot tables. Idempotent (`IF NOT EXISTS`), run by
/// `LibraryDatabase::open` next to the rest of the schema.
pub fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS qobuz_playlist_snapshot (
            qobuz_playlist_id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            owner TEXT,
            track_count INTEGER,
            snapped_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS qobuz_playlist_snapshot_tracks (
            qobuz_playlist_id INTEGER NOT NULL,
            position INTEGER NOT NULL,
            track_id INTEGER NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_qobuz_playlist_snapshot_tracks
            ON qobuz_playlist_snapshot_tracks(qobuz_playlist_id, position);
        "#,
    )
}
