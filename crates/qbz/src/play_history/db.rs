//! SQLite connection management for the local play-history store.

use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use rusqlite::Connection;

static DB: OnceLock<Mutex<Option<Connection>>> = OnceLock::new();

fn db_path() -> Option<PathBuf> {
    Some(dirs::data_dir()?.join("qbz").join("play_history.db"))
}

fn open_db() -> Option<Connection> {
    let path = db_path()?;
    if let Some(parent) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            log::warn!("[qbz-slint] play_history dir create failed: {e}");
            return None;
        }
    }
    let conn = match Connection::open(&path) {
        Ok(c) => c,
        Err(e) => {
            log::warn!("[qbz-slint] play_history open failed: {e}");
            return None;
        }
    };
    // ADR-002: WAL mode for any SQLite store touched off the UI thread.
    if let Err(e) =
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
    {
        log::warn!("[qbz-slint] play_history pragma failed: {e}");
    }
    if let Err(e) = conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS play_events (
            artist_id INTEGER NOT NULL,
            occurred_at INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS play_events_artist
            ON play_events(artist_id);

        CREATE TABLE IF NOT EXISTS artist_names (
            artist_id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            updated_at INTEGER NOT NULL
        );
        "#,
    ) {
        log::warn!("[qbz-slint] play_history schema failed: {e}");
        return None;
    }
    Some(conn)
}

pub(super) fn with_db<F, T>(f: F) -> Option<T>
where
    F: FnOnce(&Connection) -> Option<T>,
{
    let cell = DB.get_or_init(|| Mutex::new(open_db()));
    let guard = cell.lock().ok()?;
    let conn = guard.as_ref()?;
    f(conn)
}
