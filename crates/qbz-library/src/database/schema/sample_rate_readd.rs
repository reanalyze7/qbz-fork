//! Re-add half of the sample_rate INTEGER -> REAL migration (called from
//! `migrations_v4::migrate_v4`, after `sample_rate_rebuild.rs`). The
//! table rebuild only copies `local_tracks`'s core columns, so any
//! optional columns added by earlier migrations (album grouping, source
//! tracking, catalog number) are re-added here if missing. Copied
//! verbatim from the monolithic `database.rs`.

use crate::LibraryError;

use super::super::LibraryDatabase;

impl LibraryDatabase {
    pub(super) fn readd_columns_after_sample_rate_migration(&self) -> Result<(), LibraryError> {
        // Add optional columns if they existed in old table
        // These were added in previous migrations, so they may or may not exist
        let has_album_group_key: bool = self.conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('local_tracks') WHERE name = 'album_group_key'",
                [],
                |row| row.get::<_, i32>(0),
            )
            .map(|count| count > 0)
            .unwrap_or(false);

        if !has_album_group_key {
            // Re-add album grouping columns (will be populated by next migration check)
            self.conn.execute_batch(
                "ALTER TABLE local_tracks ADD COLUMN album_group_key TEXT;
                     ALTER TABLE local_tracks ADD COLUMN album_group_title TEXT;
                     CREATE INDEX IF NOT EXISTS idx_tracks_album_group ON local_tracks(album_group_key);"
            ).map_err(|e| LibraryError::Database(format!("Failed to re-add album_group columns: {}", e)))?;
        }
        // else: columns existed, meaning they survived the rebuild (should not
        // happen since the rebuild only copies core columns, but kept for safety).

        // Re-add source and qobuz_track_id columns if they don't exist
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
            self.conn.execute_batch(
                "ALTER TABLE local_tracks ADD COLUMN source TEXT DEFAULT 'user';
                     ALTER TABLE local_tracks ADD COLUMN qobuz_track_id INTEGER;
                     CREATE INDEX IF NOT EXISTS idx_tracks_source ON local_tracks(source);
                     CREATE INDEX IF NOT EXISTS idx_tracks_qobuz_id ON local_tracks(qobuz_track_id);"
            ).map_err(|e| LibraryError::Database(format!("Failed to re-add source columns: {}", e)))?;
        }

        // Re-add catalog_number if it doesn't exist
        let has_catalog: bool = self.conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('local_tracks') WHERE name = 'catalog_number'",
                [],
                |row| row.get::<_, i32>(0),
            )
            .map(|count| count > 0)
            .unwrap_or(false);

        if !has_catalog {
            self.conn
                .execute_batch("ALTER TABLE local_tracks ADD COLUMN catalog_number TEXT;")
                .map_err(|e| {
                    LibraryError::Database(format!("Failed to re-add catalog_number: {}", e))
                })?;
        }

        Ok(())
    }
}
