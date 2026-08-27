//! Schema/migrations for the local-playlist tables.

use rusqlite::{Connection, Result};

/// Create the local-playlist tables. Idempotent (`IF NOT EXISTS`), run by
/// `LibraryDatabase::open` next to the rest of the schema.
pub fn init_schema(conn: &Connection) -> Result<()> {
    // `local_playlists.folder_id` references the shared sidebar folders table
    // owned by `LibraryDatabase`. Create it here too so this module's schema
    // is self-contained (unit tests open an in-memory DB and only call
    // `init_schema`; production already creates `playlist_folders` first).
    conn.execute_batch(
        r#"
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

        CREATE TABLE IF NOT EXISTS local_playlists (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            offline_only INTEGER NOT NULL DEFAULT 0,
            favorite INTEGER NOT NULL DEFAULT 0,
            hidden INTEGER NOT NULL DEFAULT 0,
            custom_artwork_path TEXT,
            -- Sidebar folder membership. Points at the SHARED playlist_folders
            -- table (the same folders Qobuz playlists use); folder org is a
            -- Qoqobuz-side concept, so local playlists join the same folders. Local
            -- ids are strings, so they could never live in playlist_settings
            -- (u64 PK) — the membership rides here instead.
            folder_id TEXT REFERENCES playlist_folders(id) ON DELETE SET NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS local_playlist_tracks (
            playlist_id TEXT NOT NULL REFERENCES local_playlists(id) ON DELETE CASCADE,
            position INTEGER NOT NULL,
            source TEXT NOT NULL,           -- 'qobuz' | 'local'
            qobuz_track_id INTEGER,
            local_path TEXT,
            added_at INTEGER NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_local_playlist_tracks_playlist
            ON local_playlist_tracks(playlist_id, position);
        "#,
    )?;
    // Additive migration (B3): DBs created before the favorite/hidden
    // columns existed. Pragma-guarded ALTER, the database.rs idiom.
    let has_favorite: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('local_playlists') WHERE name = 'favorite'",
            [],
            |r| r.get::<_, i32>(0),
        )
        .map(|count| count > 0)
        .unwrap_or(false);
    if !has_favorite {
        conn.execute_batch(
            "ALTER TABLE local_playlists ADD COLUMN favorite INTEGER NOT NULL DEFAULT 0;
             ALTER TABLE local_playlists ADD COLUMN hidden INTEGER NOT NULL DEFAULT 0;",
        )?;
    }
    // Additive migration (folder membership): DBs created before the folder_id
    // column. The REFERENCES clause is accepted by ALTER because the default is
    // NULL and the app's connections don't enable the foreign_keys pragma.
    let has_folder: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('local_playlists') WHERE name = 'folder_id'",
            [],
            |r| r.get::<_, i32>(0),
        )
        .map(|count| count > 0)
        .unwrap_or(false);
    if !has_folder {
        conn.execute_batch(
            "ALTER TABLE local_playlists ADD COLUMN folder_id TEXT \
             REFERENCES playlist_folders(id) ON DELETE SET NULL;",
        )?;
    }
    Ok(())
}
