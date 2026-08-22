//! Album/artwork/download/kv schema. Third of three `init_schema` DDL
//! groups, split out only to stay under the 130-line file limit — copied
//! verbatim from the monolithic `database.rs`.

use crate::LibraryError;

use super::super::LibraryDatabase;

impl LibraryDatabase {
    pub(super) fn init_misc(&self) -> Result<(), LibraryError> {
        self.conn
            .execute_batch(
                r#"
            -- Album settings (per-album customization)
            CREATE TABLE IF NOT EXISTS album_settings (
                album_group_key TEXT PRIMARY KEY,
                hidden INTEGER DEFAULT 0,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );

            -- Artist images cache (Qobuz/Discogs images and custom uploads)
            CREATE TABLE IF NOT EXISTS artist_images (
                artist_name TEXT PRIMARY KEY,
                image_url TEXT,
                source TEXT NOT NULL,
                custom_image_path TEXT,
                fetched_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_artist_images_fetched ON artist_images(fetched_at);

            -- Custom album covers (user-uploaded covers for Qobuz albums)
            CREATE TABLE IF NOT EXISTS custom_album_covers (
                album_id TEXT PRIMARY KEY,
                custom_image_path TEXT NOT NULL,
                created_at INTEGER NOT NULL
            );

            -- Downloaded purchases registry (permanent — user owns these files)
            CREATE TABLE IF NOT EXISTS downloaded_purchases (
                track_id INTEGER NOT NULL,
                format_id INTEGER NOT NULL DEFAULT 0,
                album_id TEXT,
                file_path TEXT NOT NULL,
                downloaded_at TEXT NOT NULL DEFAULT (datetime('now')),
                PRIMARY KEY (track_id, format_id)
            );

            CREATE INDEX IF NOT EXISTS idx_downloaded_purchases_album
                ON downloaded_purchases(album_id);

            CREATE TABLE IF NOT EXISTS library_kv (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
        "#,
            )
            .map_err(|e| LibraryError::Database(format!("Failed to create schema: {}", e)))?;

        Ok(())
    }
}
