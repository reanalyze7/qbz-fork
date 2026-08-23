use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use rusqlite::Connection;

static DB: OnceLock<Mutex<Option<Connection>>> = OnceLock::new();

fn db_path() -> Option<PathBuf> {
    Some(dirs::data_dir()?.join("qbz").join("album_play_history.db"))
}

/// Create the tables + index on a fresh connection (shared by the lazy opener
/// and the in-memory test connections).
pub(crate) fn init_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS album_play_events (
            album_id TEXT NOT NULL,
            occurred_at INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS album_play_events_album
            ON album_play_events(album_id);

        CREATE TABLE IF NOT EXISTS album_meta (
            album_id      TEXT PRIMARY KEY,
            title         TEXT NOT NULL,
            artist        TEXT NOT NULL,
            artist_id     TEXT NOT NULL DEFAULT '',
            artwork_url   TEXT NOT NULL DEFAULT '',
            quality_tier  TEXT NOT NULL DEFAULT '',
            quality_label TEXT NOT NULL DEFAULT '',
            year          TEXT NOT NULL DEFAULT '',
            source        TEXT NOT NULL DEFAULT '',
            updated_at    INTEGER NOT NULL
        );
        "#,
    )
}

fn open_db() -> Option<Connection> {
    let path = db_path()?;
    if let Some(parent) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            log::warn!("[qbz-slint] album_play_history dir create failed: {e}");
            return None;
        }
    }
    let conn = match Connection::open(&path) {
        Ok(c) => c,
        Err(e) => {
            log::warn!("[qbz-slint] album_play_history open failed: {e}");
            return None;
        }
    };
    // ADR-002: WAL for any SQLite store touched off the UI thread.
    if let Err(e) = conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;") {
        log::warn!("[qbz-slint] album_play_history pragma failed: {e}");
    }
    if let Err(e) = init_schema(&conn) {
        log::warn!("[qbz-slint] album_play_history schema failed: {e}");
        return None;
    }
    Some(conn)
}

pub(crate) fn with_db<F, T>(f: F) -> Option<T>
where
    F: FnOnce(&Connection) -> Option<T>,
{
    let cell = DB.get_or_init(|| Mutex::new(open_db()));
    let guard = cell.lock().ok()?;
    let conn = guard.as_ref()?;
    f(conn)
}
