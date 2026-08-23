use super::types::{AutoplayMode, PlaybackPreferences};
use log::info;
use rusqlite::Connection;
use std::path::Path;

pub struct PlaybackPreferencesStore {
    pub(super) conn: Connection,
}

impl PlaybackPreferencesStore {
    fn open_at(dir: &Path, db_name: &str) -> Result<Self, String> {
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("Failed to create data directory: {}", e))?;

        let db_path = dir.join(db_name);
        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open playback preferences database: {}", e))?;

        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
            .map_err(|e| {
                format!(
                    "Failed to enable WAL for playback preferences database: {}",
                    e
                )
            })?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS playback_preferences (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                autoplay_mode TEXT NOT NULL DEFAULT 'continue'
            );",
        )
        .map_err(|e| format!("Failed to create playback preferences table: {}", e))?;

        let show_context_icon_exists =
            column_exists(&conn, "playback_preferences", "show_context_icon");
        info!(
            "[PlaybackPrefs] Column show_context_icon exists: {}",
            show_context_icon_exists
        );
        if !show_context_icon_exists {
            info!("[PlaybackPrefs] Migrating: adding show_context_icon column");
            conn.execute(
                "ALTER TABLE playback_preferences ADD COLUMN show_context_icon INTEGER NOT NULL DEFAULT 1",
                [],
            )
            .map_err(|e| format!("Failed to add show_context_icon column: {}", e))?;
            info!("[PlaybackPrefs] Migration successful");
        }

        if !column_exists(&conn, "playback_preferences", "persist_session") {
            info!("[PlaybackPrefs] Migrating: adding persist_session column");
            conn.execute(
                "ALTER TABLE playback_preferences ADD COLUMN persist_session INTEGER NOT NULL DEFAULT 1",
                [],
            )
            .map_err(|e| format!("Failed to add persist_session column: {}", e))?;
            info!("[PlaybackPrefs] persist_session migration successful");
        }

        if !column_exists(&conn, "playback_preferences", "resume_playback_position") {
            info!("[PlaybackPrefs] Migrating: adding resume_playback_position column");
            conn.execute(
                "ALTER TABLE playback_preferences ADD COLUMN resume_playback_position INTEGER NOT NULL DEFAULT 1",
                [],
            )
            .map_err(|e| format!("Failed to add resume_playback_position column: {}", e))?;
            info!("[PlaybackPrefs] resume_playback_position migration successful");
        }

        conn.execute(
            "INSERT OR IGNORE INTO playback_preferences (id, autoplay_mode, show_context_icon, persist_session, resume_playback_position)
            VALUES (1, 'continue', 1, 1, 1)",
            [],
        )
        .map_err(|e| format!("Failed to insert default preferences: {}", e))?;

        Ok(Self { conn })
    }

    pub fn new() -> Result<Self, String> {
        let data_dir = dirs::data_dir()
            .ok_or("Could not determine data directory")?
            .join("qbz");
        Self::open_at(&data_dir, "playback_preferences.db")
    }

    pub fn new_at(base_dir: &Path) -> Result<Self, String> {
        Self::open_at(base_dir, "playback_preferences.db")
    }

    pub fn get_preferences(&self) -> Result<PlaybackPreferences, String> {
        self.conn
            .query_row(
                "SELECT autoplay_mode, show_context_icon, persist_session, resume_playback_position FROM playback_preferences WHERE id = 1",
                [],
                |row| {
                    let autoplay_str: String = row.get(0)?;
                    let show_icon: i32 = row.get(1)?;
                    let persist: i32 = row.get(2)?;
                    let resume_pos: i32 = row.get(3)?;
                    Ok(PlaybackPreferences {
                        autoplay_mode: AutoplayMode::from_db_value(&autoplay_str),
                        show_context_icon: show_icon != 0,
                        persist_session: persist != 0,
                        resume_playback_position: resume_pos != 0,
                    })
                },
            )
            .map_err(|e| format!("Failed to get playback preferences: {}", e))
    }
}

fn column_exists(conn: &Connection, table: &str, column: &str) -> bool {
    conn.query_row(
        &format!("SELECT COUNT(*) FROM pragma_table_info('{table}') WHERE name = ?1"),
        [column],
        |row| {
            let count: i32 = row.get(0)?;
            Ok(count > 0)
        },
    )
    .unwrap_or(false)
}
