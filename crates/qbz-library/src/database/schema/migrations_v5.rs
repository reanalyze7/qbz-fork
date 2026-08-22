//! Migrations, part 5 of 6 (chronological order preserved). Copied
//! verbatim from the monolithic `database.rs`'s `run_migrations`; split
//! only to stay under the 130-line file limit — see `schema/mod.rs` for
//! the call order across all files.

use crate::LibraryError;

use super::super::LibraryDatabase;

impl LibraryDatabase {
    pub(super) fn migrate_v5(&self) -> Result<(), LibraryError> {
        // Migration: Add is_network_mount flag to local_tracks. Default
        // 0; callers can re-scan folders to populate real values for
        // existing rows.
        let has_network_mount: bool = self.conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('local_tracks') WHERE name = 'is_network_mount'",
                [],
                |row| row.get::<_, i32>(0),
            )
            .map(|count| count > 0)
            .unwrap_or(false);

        if !has_network_mount {
            log::info!("Running migration: adding is_network_mount to local_tracks");
            self.conn
                .execute_batch(
                    "ALTER TABLE local_tracks ADD COLUMN is_network_mount INTEGER NOT NULL DEFAULT 0;",
                )
                .map_err(|e| LibraryError::Database(format!("Migration failed: {}", e)))?;
        }

        // Migration: Add canonical_name column to artist_images for artist name normalization
        let has_canonical_name: bool = self.conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('artist_images') WHERE name = 'canonical_name'",
                [],
                |row| row.get::<_, i32>(0),
            )
            .map(|count| count > 0)
            .unwrap_or(false);

        if !has_canonical_name {
            log::info!("Running migration: adding canonical_name to artist_images");
            self.conn
                .execute_batch("ALTER TABLE artist_images ADD COLUMN canonical_name TEXT;")
                .map_err(|e| LibraryError::Database(format!("Migration failed: {}", e)))?;
        }

        // Create folder_id index after all migrations have run (ensures column exists)
        self.conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_playlist_settings_folder ON playlist_settings(folder_id);"
        ).map_err(|e| LibraryError::Database(format!("Failed to create folder index: {}", e)))?;

        Ok(())
    }
}
