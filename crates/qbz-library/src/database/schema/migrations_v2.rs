//! Migrations, part 2 of 6 (chronological order preserved). Copied
//! verbatim from the monolithic `database.rs`'s `run_migrations`; split
//! only to stay under the 130-line file limit — see `schema/mod.rs` for
//! the call order across all files.

use crate::LibraryError;

use super::super::LibraryDatabase;

impl LibraryDatabase {
    pub(super) fn migrate_v2(&self) -> Result<(), LibraryError> {
        // Migration: Add has_local_content column to playlist_settings
        let has_local_content: bool = self.conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('playlist_settings') WHERE name = 'has_local_content'",
                [],
                |row| row.get::<_, i32>(0),
            )
            .map(|count| count > 0)
            .unwrap_or(false);

        if !has_local_content {
            log::info!("Running migration: adding has_local_content column to playlist_settings");
            self.conn.execute_batch(
                "ALTER TABLE playlist_settings ADD COLUMN has_local_content TEXT DEFAULT 'unknown';
                 CREATE INDEX IF NOT EXISTS idx_playlist_local_content ON playlist_settings(has_local_content);"
            ).map_err(|e| LibraryError::Database(format!("Migration failed: {}", e)))?;
        }

        let has_file_nocue_index: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name='idx_tracks_file_nocue'",
                [],
                |row| row.get::<_, i32>(0),
            )
            .map(|count| count > 0)
            .unwrap_or(false);

        if !has_file_nocue_index {
            log::warn!("Skipping deduplication migration to prevent data loss");
            log::info!("Creating unique index for non-CUE tracks (INSERT OR REPLACE will handle duplicates)");
            // CHANGED: Don't delete duplicates automatically - let INSERT OR REPLACE handle it
            // This prevents accidental data loss from aggressive deduplication
            self.conn
                .execute_batch(
                    r#"
                CREATE UNIQUE INDEX IF NOT EXISTS idx_tracks_file_nocue
                  ON local_tracks(file_path)
                  WHERE cue_file_path IS NULL;
            "#,
                )
                .map_err(|e| LibraryError::Database(format!("Migration failed: {}", e)))?;
        }

        // Migration: Add folder metadata columns (alias, network info)
        let has_folder_alias: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('library_folders') WHERE name = 'alias'",
                [],
                |row| row.get::<_, i32>(0),
            )
            .map(|count| count > 0)
            .unwrap_or(false);

        if !has_folder_alias {
            log::info!("Running migration: adding folder metadata columns (alias, network info)");
            self.conn
                .execute_batch(
                    "ALTER TABLE library_folders ADD COLUMN alias TEXT;
                 ALTER TABLE library_folders ADD COLUMN is_network INTEGER DEFAULT 0;
                 ALTER TABLE library_folders ADD COLUMN network_fs_type TEXT;
                 ALTER TABLE library_folders ADD COLUMN user_override_network INTEGER DEFAULT 0;",
                )
                .map_err(|e| LibraryError::Database(format!("Migration failed: {}", e)))?;
        }

        Ok(())
    }
}
