use super::OfflineModeStore;
use rusqlite::Connection;
use std::path::Path;

impl OfflineModeStore {
    /// Open (or create) `offline_settings.db` under `base_dir` — the per-user
    /// data directory, same location Tauri's `OfflineState::init_at` uses.
    pub fn new_at(base_dir: &Path) -> Result<Self, String> {
        std::fs::create_dir_all(base_dir)
            .map_err(|e| format!("Failed to create data directory: {}", e))?;
        let db_path = base_dir.join("offline_settings.db");

        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open offline settings database: {}", e))?;

        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
            .map_err(|e| format!("Failed to enable WAL for offline settings database: {}", e))?;

        // Base tables — kept IDENTICAL to the Tauri module so both frontends
        // can open the same per-user file.
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS offline_settings (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                manual_offline_mode INTEGER NOT NULL DEFAULT 0,
                show_partial_playlists INTEGER NOT NULL DEFAULT 1
            );
            INSERT OR IGNORE INTO offline_settings (id, manual_offline_mode, show_partial_playlists)
            VALUES (1, 0, 1);

            CREATE TABLE IF NOT EXISTS pending_playlist_sync (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                description TEXT,
                is_public INTEGER NOT NULL DEFAULT 0,
                track_ids TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                synced INTEGER NOT NULL DEFAULT 0,
                qobuz_playlist_id INTEGER
            );
            CREATE INDEX IF NOT EXISTS idx_pending_playlist_synced ON pending_playlist_sync(synced);

            CREATE TABLE IF NOT EXISTS scrobble_queue (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                artist TEXT NOT NULL,
                track TEXT NOT NULL,
                album TEXT,
                timestamp INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                sent INTEGER NOT NULL DEFAULT 0
            );
            CREATE INDEX IF NOT EXISTS idx_scrobble_queue_sent ON scrobble_queue(sent);",
        )
        .map_err(|e| format!("Failed to create offline settings table: {}", e))?;

        // Additive migrations — same list as Tauri's; errors ignored because
        // the column may already exist.
        let migrations = [
            "ALTER TABLE offline_settings ADD COLUMN allow_cast_while_offline INTEGER NOT NULL DEFAULT 0",
            "ALTER TABLE offline_settings ADD COLUMN allow_immediate_scrobbling INTEGER NOT NULL DEFAULT 0",
            "ALTER TABLE offline_settings ADD COLUMN allow_accumulated_scrobbling INTEGER NOT NULL DEFAULT 1",
            "ALTER TABLE offline_settings ADD COLUMN show_network_folders_in_manual_offline INTEGER NOT NULL DEFAULT 0",
            "ALTER TABLE offline_settings ADD COLUMN pre_offline_stream_first_track INTEGER",
            "ALTER TABLE pending_playlist_sync ADD COLUMN local_track_ids TEXT",
            "ALTER TABLE pending_playlist_sync ADD COLUMN local_track_paths TEXT",
            "ALTER TABLE offline_settings ADD COLUMN cache_limit_bytes INTEGER",
        ];
        for migration in migrations {
            let _ = conn.execute(migration, []);
        }

        Ok(Self { conn })
    }
}
