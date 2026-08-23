use rusqlite::Connection;
use std::path::Path;

use super::migrations::run_migrations;

pub struct SessionStore {
    pub(super) conn: Connection,
}

impl SessionStore {
    pub fn new() -> Result<Self, String> {
        let data_dir = dirs::data_dir()
            .ok_or("Could not determine data directory")?
            .join("qbz");
        Self::open_at(&data_dir, "session.db")
    }

    pub fn new_at(base_dir: &Path) -> Result<Self, String> {
        Self::open_at(base_dir, "session.db")
    }

    pub(super) fn open_at(dir: &Path, db_name: &str) -> Result<Self, String> {
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("Failed to create data directory: {}", e))?;

        let db_path = dir.join(db_name);
        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open session database: {}", e))?;

        // WAL mode for non-blocking reads/writes (ADR-002). synchronous=FULL,
        // not NORMAL: the session DB must survive hard reboots (issue #440).
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL;")
            .map_err(|e| format!("Failed to set WAL mode: {}", e))?;

        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS player_state (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                current_index INTEGER,
                current_position_secs INTEGER NOT NULL DEFAULT 0,
                volume REAL NOT NULL DEFAULT 0.75,
                shuffle_enabled INTEGER NOT NULL DEFAULT 0,
                repeat_mode TEXT NOT NULL DEFAULT 'off',
                was_playing INTEGER NOT NULL DEFAULT 0,
                saved_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS queue_tracks (
                position INTEGER PRIMARY KEY,
                track_id INTEGER NOT NULL,
                title TEXT NOT NULL,
                artist TEXT NOT NULL,
                album TEXT NOT NULL,
                duration_secs INTEGER NOT NULL,
                artwork_url TEXT,
                hires INTEGER NOT NULL DEFAULT 0,
                bit_depth INTEGER,
                sample_rate REAL,
                source TEXT
            );

            INSERT OR IGNORE INTO player_state (id, current_position_secs, volume, shuffle_enabled, repeat_mode, was_playing, saved_at)
            VALUES (1, 0, 0.75, 0, 'off', 0, 0);
            ",
        )
        .map_err(|e| format!("Failed to create session tables: {}", e))?;

        run_migrations(&conn);

        Ok(Self { conn })
    }
}
