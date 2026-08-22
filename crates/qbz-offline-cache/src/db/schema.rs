//! `OfflineCacheDb` struct definition, connection setup, schema creation,
//! and the additive v2-CMAF column migration.

use rusqlite::Connection;
use std::path::Path;

/// Database wrapper for cached tracks index
pub struct OfflineCacheDb {
    pub(super) conn: Connection,
}

impl OfflineCacheDb {
    /// Open or create the database
    pub fn new(path: &Path) -> Result<Self, String> {
        let conn = Connection::open(path)
            .map_err(|e| format!("Failed to open offline cache database: {}", e))?;

        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
            .map_err(|e| format!("Failed to enable WAL for offline cache database: {}", e))?;

        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    /// Get reference to the connection (for direct queries)
    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    /// Initialize database schema
    fn init_schema(&self) -> Result<(), String> {
        self.conn
            .execute_batch(
                "
            CREATE TABLE IF NOT EXISTS cached_tracks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                track_id INTEGER UNIQUE NOT NULL,
                title TEXT NOT NULL,
                artist TEXT NOT NULL,
                album TEXT,
                album_id TEXT,
                duration_secs INTEGER NOT NULL,
                file_path TEXT NOT NULL,
                file_size_bytes INTEGER NOT NULL DEFAULT 0,
                format TEXT NOT NULL DEFAULT 'flac',
                quality TEXT,
                bit_depth INTEGER,
                sample_rate REAL,
                artwork_path TEXT,
                status TEXT NOT NULL DEFAULT 'queued',
                progress_percent INTEGER NOT NULL DEFAULT 0,
                error_message TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                last_accessed_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE INDEX IF NOT EXISTS idx_track_id ON cached_tracks(track_id);
            CREATE INDEX IF NOT EXISTS idx_status ON cached_tracks(status);
            CREATE INDEX IF NOT EXISTS idx_last_accessed ON cached_tracks(last_accessed_at);
            ",
            )
            .map_err(|e| format!("Failed to initialize database schema: {}", e))?;

        self.migrate_v2_cmaf_columns()?;

        Ok(())
    }

    /// Additive migration for the v2 offline format.
    ///
    /// Adds columns for bit-identical CMAF storage:
    /// - `cache_format`: 1 = legacy plain FLAC, 2 = raw CMAF bundle
    /// - `init_path`: path to the init.mp4 (contains FLAC header + table)
    /// - `content_key_wrapped`: AES content key wrapped with qbz-secrets
    /// - `infos_wrapped`: session infos salt wrapped with qbz-secrets
    /// - `format_id`: Qobuz format id (e.g. 5/6/7/27)
    /// - `n_segments`: number of audio segments (s=1..=n)
    ///
    /// Existing rows keep `cache_format=1` so playback continues to read
    /// the legacy plain-FLAC `file_path` for them. New downloads go to
    /// `cache_format=2`. We never rewrite v1 rows into v2 — the two
    /// formats coexist until the v1 rows naturally expire via the
    /// subscription-lapse cache wipe or user-triggered re-download.
    fn migrate_v2_cmaf_columns(&self) -> Result<(), String> {
        let existing = self.existing_columns("cached_tracks")?;
        let add = |col: &str, ddl: &str| -> Result<(), String> {
            if !existing.iter().any(|c| c == col) {
                let sql = format!("ALTER TABLE cached_tracks ADD COLUMN {}", ddl);
                self.conn
                    .execute(&sql, [])
                    .map_err(|e| format!("Failed to add column {}: {}", col, e))?;
                log::info!(
                    "[OfflineCache/MIGRATE] Added column {} to cached_tracks",
                    col
                );
            }
            Ok(())
        };
        add("cache_format", "cache_format INTEGER NOT NULL DEFAULT 1")?;
        add("init_path", "init_path TEXT")?;
        add("content_key_wrapped", "content_key_wrapped BLOB")?;
        add("infos_wrapped", "infos_wrapped BLOB")?;
        add("format_id", "format_id INTEGER")?;
        add("n_segments", "n_segments INTEGER")?;
        Ok(())
    }

    fn existing_columns(&self, table: &str) -> Result<Vec<String>, String> {
        let mut stmt = self
            .conn
            .prepare(&format!("PRAGMA table_info({})", table))
            .map_err(|e| format!("Failed to prepare PRAGMA: {}", e))?;
        let cols = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .map_err(|e| format!("Failed to read PRAGMA: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to iterate PRAGMA: {}", e))?;
        Ok(cols)
    }
}
