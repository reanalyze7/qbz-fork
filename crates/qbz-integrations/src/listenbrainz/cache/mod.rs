//! ListenBrainz cache for offline listen queue
//!
//! SQLite-based persistence for:
//! - User credentials (token, username)
//! - Queued listens for offline submission
//! - Enabled state

use rusqlite::Connection;
use std::path::Path;

mod credentials;
mod queue_read;
mod queue_write;

/// ListenBrainz cache for offline support
pub struct ListenBrainzCache {
    conn: Connection,
}

impl ListenBrainzCache {
    /// Create a new cache at the given path
    pub fn new(db_path: &Path) -> Result<Self, String> {
        let conn = Connection::open(db_path)
            .map_err(|e| format!("Failed to open ListenBrainz cache: {}", e))?;

        // Enable WAL mode for concurrent read/write (ADR-002)
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
            .map_err(|e| format!("Failed to enable WAL mode: {}", e))?;

        let cache = Self { conn };
        cache.init_schema()?;

        Ok(cache)
    }

    fn init_schema(&self) -> Result<(), String> {
        self.conn
            .execute_batch(
                "
                CREATE TABLE IF NOT EXISTS credentials (
                    id INTEGER PRIMARY KEY CHECK (id = 1),
                    token TEXT,
                    user_name TEXT
                );

                CREATE TABLE IF NOT EXISTS settings (
                    key TEXT PRIMARY KEY,
                    value TEXT NOT NULL
                );

                CREATE TABLE IF NOT EXISTS listen_queue (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    listened_at INTEGER NOT NULL,
                    artist_name TEXT NOT NULL,
                    track_name TEXT NOT NULL,
                    release_name TEXT,
                    recording_mbid TEXT,
                    release_mbid TEXT,
                    artist_mbids TEXT,
                    isrc TEXT,
                    duration_ms INTEGER,
                    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
                    attempts INTEGER DEFAULT 0,
                    sent INTEGER DEFAULT 0
                );

                CREATE INDEX IF NOT EXISTS idx_listen_queue_sent ON listen_queue(sent);
            ",
            )
            .map_err(|e| format!("Failed to init ListenBrainz schema: {}", e))
    }
}
