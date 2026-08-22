//! Playlist schema. Second of three `init_schema` DDL groups, split out
//! only to stay under the 130-line file limit — copied verbatim from the
//! monolithic `database.rs`. See also `init_misc` for the remaining
//! album/artwork/download/kv tables.

use crate::LibraryError;

use super::super::LibraryDatabase;

impl LibraryDatabase {
    pub(super) fn init_extra(&self) -> Result<(), LibraryError> {
        self.conn
            .execute_batch(
                r#"
            -- Playlist folders (local organization for Qobuz playlists)
            CREATE TABLE IF NOT EXISTS playlist_folders (
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
            CREATE INDEX IF NOT EXISTS idx_playlist_folders_hidden ON playlist_folders(is_hidden);

            -- Playlist local settings (enhances remote Qobuz playlists)
            -- Note: For existing databases, folder_id is added via migration
            CREATE TABLE IF NOT EXISTS playlist_settings (
                qobuz_playlist_id INTEGER PRIMARY KEY,
                custom_artwork_path TEXT,
                sort_by TEXT DEFAULT 'default',
                sort_order TEXT DEFAULT 'asc',
                last_search_query TEXT,
                notes TEXT,
                hidden INTEGER DEFAULT 0,
                position INTEGER DEFAULT 0,
                folder_id TEXT REFERENCES playlist_folders(id) ON DELETE SET NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );

            -- Note: idx_playlist_settings_folder is created conditionally after migrations run

            -- Playlist statistics (play counts, etc.)
            CREATE TABLE IF NOT EXISTS playlist_stats (
                qobuz_playlist_id INTEGER PRIMARY KEY,
                play_count INTEGER DEFAULT 0,
                last_played_at INTEGER,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );

            -- Qobuz playlists the user has COPIED into their library (mirrors
            -- Tauri's user-scoped `qbz_copied_playlists`): stores the SOURCE
            -- playlist id so its detail view hides the Copy button on reopen.
            CREATE TABLE IF NOT EXISTS copied_playlists (
                qobuz_playlist_id INTEGER PRIMARY KEY,
                copied_at INTEGER NOT NULL
            );

            -- Local tracks added to playlists (mixed with remote Qobuz tracks)
            CREATE TABLE IF NOT EXISTS playlist_local_tracks (
                id INTEGER PRIMARY KEY,
                qobuz_playlist_id INTEGER NOT NULL,
                local_track_id INTEGER NOT NULL,
                position INTEGER NOT NULL,
                added_at INTEGER NOT NULL,
                FOREIGN KEY (local_track_id) REFERENCES local_tracks(id) ON DELETE CASCADE,
                UNIQUE(qobuz_playlist_id, local_track_id)
            );

            CREATE INDEX IF NOT EXISTS idx_playlist_local_tracks_playlist
                ON playlist_local_tracks(qobuz_playlist_id);

            -- Custom track order per playlist (user-defined arrangement)
            CREATE TABLE IF NOT EXISTS playlist_track_custom_order (
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
                ON playlist_track_custom_order(qobuz_playlist_id, custom_position);
        "#,
            )
            .map_err(|e| LibraryError::Database(format!("Failed to create schema: {}", e)))?;

        Ok(())
    }
}
