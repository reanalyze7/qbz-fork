use super::DeveloperSettings;
use rusqlite::{params, Connection};
use std::path::Path;

const DB_FILE: &str = "developer_settings.db";

pub struct DeveloperSettingsStore {
    conn: Connection,
}

impl DeveloperSettingsStore {
    /// Lightweight read-only open for startup before host-managed state exists.
    /// Opens existing DB without creating tables or running migrations.
    pub fn new_readonly() -> Result<Self, String> {
        let db_path = dirs::data_dir()
            .ok_or("Could not determine data directory")?
            .join("qbz")
            .join(DB_FILE);
        Self::new_readonly_at_path(&db_path)
    }

    pub fn new_readonly_at_path(db_path: &Path) -> Result<Self, String> {
        if !db_path.exists() {
            return Err("Developer settings DB does not exist yet".to_string());
        }

        let conn = Connection::open_with_flags(
            db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|e| {
            format!(
                "Failed to open developer settings database (readonly): {}",
                e
            )
        })?;

        Ok(Self { conn })
    }

    fn open_at(dir: &Path, db_name: &str) -> Result<Self, String> {
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("Failed to create data directory: {}", e))?;

        let db_path = dir.join(db_name);
        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open developer settings database: {}", e))?;

        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
            .map_err(|e| {
                format!(
                    "Failed to enable WAL for developer settings database: {}",
                    e
                )
            })?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS developer_settings (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                force_dmabuf INTEGER NOT NULL DEFAULT 0
            );
            INSERT OR IGNORE INTO developer_settings (id, force_dmabuf) VALUES (1, 0);",
        )
        .map_err(|e| format!("Failed to create developer settings table: {}", e))?;

        Ok(Self { conn })
    }

    pub fn new() -> Result<Self, String> {
        let data_dir = dirs::data_dir()
            .ok_or("Could not determine data directory")?
            .join("qbz");
        Self::open_at(&data_dir, DB_FILE)
    }

    pub fn new_at(base_dir: &Path) -> Result<Self, String> {
        Self::open_at(base_dir, DB_FILE)
    }

    pub fn get_settings(&self) -> Result<DeveloperSettings, String> {
        self.conn
            .query_row(
                "SELECT force_dmabuf FROM developer_settings WHERE id = 1",
                [],
                |row| {
                    Ok(DeveloperSettings {
                        force_dmabuf: row.get::<_, i64>(0)? != 0,
                    })
                },
            )
            .map_err(|e| format!("Failed to get developer settings: {}", e))
    }

    pub fn set_force_dmabuf(&self, enabled: bool) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE developer_settings SET force_dmabuf = ?1 WHERE id = 1",
                params![enabled as i64],
            )
            .map_err(|e| format!("Failed to set force_dmabuf: {}", e))?;
        Ok(())
    }
}
