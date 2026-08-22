//! Table-rebuild half of the sample_rate INTEGER -> REAL migration
//! (called from `migrations_v4::migrate_v4`). SQLite cannot ALTER COLUMN
//! a type, so this recreates `local_tracks` with only its core columns —
//! optional columns added by later migrations are re-added afterward by
//! `sample_rate_readd.rs`. Copied verbatim from the monolithic
//! `database.rs`.

use crate::LibraryError;

use super::super::LibraryDatabase;

impl LibraryDatabase {
    pub(super) fn rebuild_local_tracks_for_real_sample_rate(&self) -> Result<(), LibraryError> {
        // SQLite doesn't support ALTER COLUMN type change, need to recreate table
        // CRITICAL: Explicitly list all columns to handle different DB versions safely
        self.conn
            .execute_batch(
                r#"
                -- Clean up any leftover temp table from previous failed migration
                DROP TABLE IF EXISTS local_tracks_new;

                -- Create new table with REAL sample_rate (only core columns)
                CREATE TABLE local_tracks_new (
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
                    UNIQUE(file_path, cue_start_secs)
                );

                -- Copy core columns explicitly (handles all DB versions)
                -- Use COALESCE to handle NULL values and provide safe defaults
                INSERT INTO local_tracks_new
                    (id, file_path, title, artist, album, album_artist, track_number,
                     disc_number, year, genre, duration_secs, format, bit_depth,
                     sample_rate, channels, file_size_bytes, cue_file_path,
                     cue_start_secs, cue_end_secs, artwork_path, last_modified, indexed_at)
                SELECT
                    id, file_path, title, artist, album,
                    album_artist, track_number, disc_number, year, genre,
                    duration_secs, format, bit_depth,
                    CAST(sample_rate AS REAL),
                    channels,
                    COALESCE(file_size_bytes, 0),
                    cue_file_path, cue_start_secs, cue_end_secs,
                    artwork_path, last_modified, indexed_at
                FROM local_tracks;

                -- Drop old table
                DROP TABLE local_tracks;

                -- Rename new table
                ALTER TABLE local_tracks_new RENAME TO local_tracks;

                -- Recreate core indexes
                CREATE INDEX IF NOT EXISTS idx_tracks_artist ON local_tracks(artist);
                CREATE INDEX IF NOT EXISTS idx_tracks_album ON local_tracks(album);
                CREATE INDEX IF NOT EXISTS idx_tracks_album_artist ON local_tracks(album_artist);
                CREATE INDEX IF NOT EXISTS idx_tracks_file_path ON local_tracks(file_path);
                CREATE INDEX IF NOT EXISTS idx_tracks_title ON local_tracks(title);
                CREATE UNIQUE INDEX IF NOT EXISTS idx_tracks_file_nocue
                    ON local_tracks(file_path)
                    WHERE cue_file_path IS NULL;
                "#,
            )
            .map_err(|e| LibraryError::Database(format!("sample_rate migration failed: {}", e)))?;

        Ok(())
    }
}
