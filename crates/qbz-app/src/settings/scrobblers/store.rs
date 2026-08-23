use super::ScrobblerSettings;
use log::info;
use rusqlite::Connection;
use std::path::Path;

pub struct ScrobblerSettingsStore {
    pub(super) conn: Connection,
}

impl ScrobblerSettingsStore {
    fn open_at(dir: &Path, db_name: &str) -> Result<Self, String> {
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("Failed to create data directory: {}", e))?;

        let db_path = dir.join(db_name);
        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open scrobbler settings database: {}", e))?;

        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
            .map_err(|e| format!("Failed to enable WAL for scrobbler settings database: {}", e))?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS scrobbler_settings (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                enabled INTEGER NOT NULL DEFAULT 0,
                ui_collapsed INTEGER NOT NULL DEFAULT 0,
                lastfm_enabled INTEGER NOT NULL DEFAULT 0,
                lastfm_session_key TEXT NOT NULL DEFAULT '',
                lastfm_username TEXT NOT NULL DEFAULT '',
                listenbrainz_enabled INTEGER NOT NULL DEFAULT 0,
                listenbrainz_token TEXT NOT NULL DEFAULT '',
                listenbrainz_username TEXT NOT NULL DEFAULT ''
            );",
        )
        .map_err(|e| format!("Failed to create scrobbler settings table: {}", e))?;

        conn.execute(
            "INSERT OR IGNORE INTO scrobbler_settings
                (id, enabled, ui_collapsed, lastfm_enabled, lastfm_session_key,
                 lastfm_username, listenbrainz_enabled, listenbrainz_token,
                 listenbrainz_username)
             VALUES (1, 0, 0, 0, '', '', 0, '', '')",
            [],
        )
        .map_err(|e| format!("Failed to insert default scrobbler settings: {}", e))?;

        info!("[ScrobblerSettings] Database initialized");

        Ok(Self { conn })
    }

    pub fn new_at(base_dir: &Path) -> Result<Self, String> {
        Self::open_at(base_dir, "scrobbler_settings.db")
    }

    pub fn get_settings(&self) -> Result<ScrobblerSettings, String> {
        self.conn
            .query_row(
                "SELECT enabled, ui_collapsed, lastfm_enabled, lastfm_session_key,
                        lastfm_username, listenbrainz_enabled, listenbrainz_token,
                        listenbrainz_username
                 FROM scrobbler_settings WHERE id = 1",
                [],
                |row| {
                    let enabled: i32 = row.get(0)?;
                    let ui_collapsed: i32 = row.get(1)?;
                    let lastfm_enabled: i32 = row.get(2)?;
                    let lastfm_session_key: String = row.get(3)?;
                    let lastfm_username: String = row.get(4)?;
                    let listenbrainz_enabled: i32 = row.get(5)?;
                    let listenbrainz_token: String = row.get(6)?;
                    let listenbrainz_username: String = row.get(7)?;
                    Ok(ScrobblerSettings {
                        enabled: enabled != 0,
                        ui_collapsed: ui_collapsed != 0,
                        lastfm_enabled: lastfm_enabled != 0,
                        lastfm_session_key,
                        lastfm_username,
                        listenbrainz_enabled: listenbrainz_enabled != 0,
                        listenbrainz_token,
                        listenbrainz_username,
                    })
                },
            )
            .map_err(|e| format!("Failed to get scrobbler settings: {}", e))
    }

    pub fn set_enabled(&self, value: bool) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE scrobbler_settings SET enabled = ?1 WHERE id = 1",
                rusqlite::params![if value { 1 } else { 0 }],
            )
            .map_err(|e| format!("Failed to set scrobbler enabled: {}", e))?;
        Ok(())
    }

    pub fn set_ui_collapsed(&self, value: bool) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE scrobbler_settings SET ui_collapsed = ?1 WHERE id = 1",
                rusqlite::params![if value { 1 } else { 0 }],
            )
            .map_err(|e| format!("Failed to set scrobbler ui_collapsed: {}", e))?;
        Ok(())
    }
}
