//! Migrations, part 6 of 6 (chronological order preserved). Copied
//! verbatim from the monolithic `database.rs`'s `run_migrations`; split
//! only to stay under the 130-line file limit — see `schema/mod.rs` for
//! the call order across all files.

use crate::LibraryError;

use super::super::LibraryDatabase;

impl LibraryDatabase {
    pub(super) fn migrate_v6(&self) -> Result<(), LibraryError> {
        // Migration: Create playlist_track_custom_order table for custom track arrangement
        let has_custom_order_table: bool = self.conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='playlist_track_custom_order'",
                [],
                |row| row.get::<_, i32>(0),
            )
            .map(|count| count > 0)
            .unwrap_or(false);

        if !has_custom_order_table {
            log::info!("Running migration: creating playlist_track_custom_order table");
            self.conn
                .execute_batch(
                    "CREATE TABLE IF NOT EXISTS playlist_track_custom_order (
                    id INTEGER PRIMARY KEY,
                    qobuz_playlist_id INTEGER NOT NULL,
                    track_id INTEGER NOT NULL,
                    is_local INTEGER DEFAULT 0,
                    custom_position INTEGER NOT NULL,
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL,
                    UNIQUE(qobuz_playlist_id, track_id, is_local)
                );
                CREATE INDEX IF NOT EXISTS idx_playlist_custom_order_playlist
                    ON playlist_track_custom_order(qobuz_playlist_id);
                CREATE INDEX IF NOT EXISTS idx_playlist_custom_order_position
                    ON playlist_track_custom_order(qobuz_playlist_id, custom_position);",
                )
                .map_err(|e| LibraryError::Database(format!("Migration failed: {}", e)))?;
        }

        // Migration: Add format_id to downloaded_purchases (compound PK: track_id + format_id)
        let has_format_id: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('downloaded_purchases') WHERE name = 'format_id'",
                [],
                |row| row.get::<_, i32>(0),
            )
            .map(|count| count > 0)
            .unwrap_or(false);

        if !has_format_id {
            log::info!("Running migration: adding format_id to downloaded_purchases (compound PK)");
            self.conn
                .execute_batch(
                    r#"
                DROP TABLE IF EXISTS downloaded_purchases_new;

                CREATE TABLE downloaded_purchases_new (
                    track_id INTEGER NOT NULL,
                    format_id INTEGER NOT NULL DEFAULT 0,
                    album_id TEXT,
                    file_path TEXT NOT NULL,
                    downloaded_at TEXT NOT NULL DEFAULT (datetime('now')),
                    PRIMARY KEY (track_id, format_id)
                );

                INSERT INTO downloaded_purchases_new (track_id, format_id, album_id, file_path, downloaded_at)
                    SELECT track_id, 0, album_id, file_path, downloaded_at
                    FROM downloaded_purchases;

                DROP TABLE downloaded_purchases;
                ALTER TABLE downloaded_purchases_new RENAME TO downloaded_purchases;

                CREATE INDEX IF NOT EXISTS idx_downloaded_purchases_album
                    ON downloaded_purchases(album_id);
                "#,
                )
                .map_err(|e| {
                    LibraryError::Database(format!(
                        "downloaded_purchases format_id migration failed: {}",
                        e
                    ))
                })?;
        }

        Ok(())
    }
}
