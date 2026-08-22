//! Migrations, part 3 of 6 (chronological order preserved). Copied
//! verbatim from the monolithic `database.rs`'s `run_migrations`; split
//! only to stay under the 130-line file limit — see `schema/mod.rs` for
//! the call order across all files.

use crate::LibraryError;

use super::super::LibraryDatabase;

impl LibraryDatabase {
    pub(super) fn migrate_v3(&self) -> Result<(), LibraryError> {
        // Migration: Add is_favorite column to playlist_settings
        let has_is_favorite: bool = self.conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('playlist_settings') WHERE name = 'is_favorite'",
                [],
                |row| row.get::<_, i32>(0),
            )
            .map(|count| count > 0)
            .unwrap_or(false);

        if !has_is_favorite {
            log::info!("Running migration: adding is_favorite column to playlist_settings");
            self.conn.execute_batch(
                "ALTER TABLE playlist_settings ADD COLUMN is_favorite INTEGER DEFAULT 0;
                 CREATE INDEX IF NOT EXISTS idx_playlist_favorite ON playlist_settings(is_favorite);"
            ).map_err(|e| LibraryError::Database(format!("Migration failed: {}", e)))?;
        }

        // Migration: Add playlist_folders table and folder_id column
        let has_playlist_folders: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='playlist_folders'",
                [],
                |row| row.get::<_, i32>(0),
            )
            .map(|count| count > 0)
            .unwrap_or(false);

        if !has_playlist_folders {
            log::info!("Running migration: creating playlist_folders table");
            self.conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS playlist_folders (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    icon_type TEXT DEFAULT 'preset',
                    icon_preset TEXT DEFAULT 'folder',
                    icon_color TEXT DEFAULT '#6366f1',
                    custom_image_path TEXT,
                    is_hidden INTEGER DEFAULT 0,
                    position INTEGER DEFAULT 0,
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_playlist_folders_position ON playlist_folders(position);
                CREATE INDEX IF NOT EXISTS idx_playlist_folders_hidden ON playlist_folders(is_hidden);"
            ).map_err(|e| LibraryError::Database(format!("Migration failed: {}", e)))?;
        }

        // Migration: Add folder_id column to playlist_settings
        let has_folder_id: bool = self.conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('playlist_settings') WHERE name = 'folder_id'",
                [],
                |row| row.get::<_, i32>(0),
            )
            .map(|count| count > 0)
            .unwrap_or(false);

        if !has_folder_id {
            log::info!("Running migration: adding folder_id column to playlist_settings");
            self.conn.execute_batch(
                "ALTER TABLE playlist_settings ADD COLUMN folder_id TEXT REFERENCES playlist_folders(id) ON DELETE SET NULL;
                 CREATE INDEX IF NOT EXISTS idx_playlist_settings_folder ON playlist_settings(folder_id);"
            ).map_err(|e| LibraryError::Database(format!("Migration failed: {}", e)))?;
        }

        // Migration: Add catalog_number column to local_tracks
        let has_catalog_number: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('local_tracks') WHERE name = 'catalog_number'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .map(|count| count > 0)
            .unwrap_or(false);

        if !has_catalog_number {
            log::info!("Running migration: adding catalog_number to local_tracks");
            self.conn
                .execute_batch("ALTER TABLE local_tracks ADD COLUMN catalog_number TEXT;")
                .map_err(|e| LibraryError::Database(format!("Migration failed: {}", e)))?;
        }

        Ok(())
    }
}
