//! Migrations, part 1 of 4 (chronological order preserved). Copied
//! verbatim from the monolithic `database.rs`'s `run_migrations`; split
//! only to stay under the 130-line file limit — see `schema/mod.rs` for
//! the call order across all four files.

use crate::LibraryError;

use super::super::LibraryDatabase;

impl LibraryDatabase {
    pub(super) fn migrate_v1(&self) -> Result<(), LibraryError> {
        // Migration: Add qobuz download tracking fields
        let has_source: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('local_tracks') WHERE name = 'source'",
                [],
                |row| row.get::<_, i32>(0),
            )
            .map(|count| count > 0)
            .unwrap_or(false);

        if !has_source {
            log::info!("Running migration: adding source and qobuz_track_id to local_tracks");
            self.conn
                .execute_batch(
                    "ALTER TABLE local_tracks ADD COLUMN source TEXT DEFAULT 'user';
                 ALTER TABLE local_tracks ADD COLUMN qobuz_track_id INTEGER;
                 CREATE INDEX IF NOT EXISTS idx_tracks_source ON local_tracks(source);
                 CREATE INDEX IF NOT EXISTS idx_tracks_qobuz_id ON local_tracks(qobuz_track_id);",
                )
                .map_err(|e| LibraryError::Database(format!("Migration failed: {}", e)))?;
        }

        // Check if playlist_settings has the 'hidden' column (added in v2)
        let has_hidden: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('playlist_settings') WHERE name = 'hidden'",
                [],
                |row| row.get::<_, i32>(0),
            )
            .map(|count| count > 0)
            .unwrap_or(false);

        if !has_hidden {
            log::info!(
                "Running migration: adding hidden and position columns to playlist_settings"
            );
            self.conn
                .execute_batch(
                    "ALTER TABLE playlist_settings ADD COLUMN hidden INTEGER DEFAULT 0;
                 ALTER TABLE playlist_settings ADD COLUMN position INTEGER DEFAULT 0;",
                )
                .map_err(|e| LibraryError::Database(format!("Migration failed: {}", e)))?;
        }

        // Check if playlist_stats table exists
        let has_stats_table: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='playlist_stats'",
                [],
                |row| row.get::<_, i32>(0),
            )
            .map(|count| count > 0)
            .unwrap_or(false);

        if !has_stats_table {
            log::info!("Running migration: creating playlist_stats table");
            self.conn
                .execute_batch(
                    "CREATE TABLE IF NOT EXISTS playlist_stats (
                    qobuz_playlist_id INTEGER PRIMARY KEY,
                    play_count INTEGER DEFAULT 0,
                    last_played_at INTEGER,
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL
                );",
                )
                .map_err(|e| LibraryError::Database(format!("Migration failed: {}", e)))?;
        }

        let has_album_group_key: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('local_tracks') WHERE name = 'album_group_key'",
                [],
                |row| row.get::<_, i32>(0),
            )
            .map(|count| count > 0)
            .unwrap_or(false);

        if !has_album_group_key {
            log::info!("Running migration: adding album_group_key to local_tracks");
            self.conn
                .execute_batch("ALTER TABLE local_tracks ADD COLUMN album_group_key TEXT;")
                .map_err(|e| LibraryError::Database(format!("Migration failed: {}", e)))?;
        }

        let has_album_group_title: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('local_tracks') WHERE name = 'album_group_title'",
                [],
                |row| row.get::<_, i32>(0),
            )
            .map(|count| count > 0)
            .unwrap_or(false);

        if !has_album_group_title {
            log::info!("Running migration: adding album_group_title to local_tracks");
            self.conn
                .execute_batch("ALTER TABLE local_tracks ADD COLUMN album_group_title TEXT;")
                .map_err(|e| LibraryError::Database(format!("Migration failed: {}", e)))?;
        }

        self.conn
            .execute_batch(
                "CREATE INDEX IF NOT EXISTS idx_tracks_album_group ON local_tracks(album_group_key);",
            )
            .map_err(|e| LibraryError::Database(format!("Migration failed: {}", e)))?;

        Ok(())
    }
}
