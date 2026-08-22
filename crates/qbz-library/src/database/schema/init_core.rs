//! Core schema: local folders and tracks. First half of `init_schema`'s
//! original DDL, split out only to stay under the 130-line file limit —
//! copied verbatim from the monolithic `database.rs`.

use crate::LibraryError;

use super::super::LibraryDatabase;

impl LibraryDatabase {
    pub(super) fn init_core(&self) -> Result<(), LibraryError> {
        self.conn
            .execute_batch(
                r#"
            CREATE TABLE IF NOT EXISTS library_folders (
                id INTEGER PRIMARY KEY,
                path TEXT UNIQUE NOT NULL,
                enabled INTEGER DEFAULT 1,
                last_scan INTEGER
            );

            CREATE TABLE IF NOT EXISTS local_tracks (
                id INTEGER PRIMARY KEY,
                file_path TEXT NOT NULL,
                title TEXT NOT NULL,
                artist TEXT NOT NULL,
                album TEXT NOT NULL,
                album_artist TEXT,
                track_number INTEGER,
                disc_number INTEGER,
                year INTEGER,
                genre TEXT,
                duration_secs INTEGER NOT NULL,
                format TEXT NOT NULL,
                bit_depth INTEGER,
                sample_rate REAL NOT NULL,
                channels INTEGER NOT NULL,
                file_size_bytes INTEGER NOT NULL,
                cue_file_path TEXT,
                cue_start_secs REAL,
                cue_end_secs REAL,
                artwork_path TEXT,
                last_modified INTEGER NOT NULL,
                indexed_at INTEGER NOT NULL,
                album_group_key TEXT,
                album_group_title TEXT,
                is_network_mount INTEGER NOT NULL DEFAULT 0,
                UNIQUE(file_path, cue_start_secs)
            );

            CREATE INDEX IF NOT EXISTS idx_tracks_artist ON local_tracks(artist);
            CREATE INDEX IF NOT EXISTS idx_tracks_album ON local_tracks(album);
            CREATE INDEX IF NOT EXISTS idx_tracks_album_artist ON local_tracks(album_artist);
            CREATE INDEX IF NOT EXISTS idx_tracks_file_path ON local_tracks(file_path);
            CREATE INDEX IF NOT EXISTS idx_tracks_title ON local_tracks(title);
            CREATE INDEX IF NOT EXISTS idx_local_tracks_album_lookup
                ON local_tracks(album, album_artist, artist);
        "#,
            )
            .map_err(|e| LibraryError::Database(format!("Failed to create schema: {}", e)))?;

        Ok(())
    }
}
