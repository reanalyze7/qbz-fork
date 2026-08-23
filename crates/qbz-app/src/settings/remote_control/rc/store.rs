use base64::Engine;
use rand::RngExt;
use rusqlite::{params, Connection};
use std::path::Path;

use super::RemoteControlSettings;

pub struct RemoteControlSettingsStore {
    pub(super) conn: Connection,
}

impl RemoteControlSettingsStore {
    fn open_at(dir: &Path, db_name: &str) -> Result<Self, String> {
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("Failed to create data directory: {}", e))?;

        let db_path = dir.join(db_name);
        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open remote control settings database: {}", e))?;

        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
            .map_err(|e| {
                format!(
                    "Failed to enable WAL for remote control settings database: {}",
                    e
                )
            })?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS remote_control_settings (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                enabled INTEGER NOT NULL DEFAULT 0,
                port INTEGER NOT NULL DEFAULT 8182,
                secure INTEGER NOT NULL DEFAULT 1,
                token TEXT NOT NULL DEFAULT ''
            );",
        )
        .map_err(|e| format!("Failed to create remote control settings table: {}", e))?;

        ensure_secure_column(&conn)?;

        let token = generate_token();
        conn.execute(
            "INSERT OR IGNORE INTO remote_control_settings (id, enabled, port, secure, token)
            VALUES (1, 0, 8182, 0, ?1)",
            params![token],
        )
        .map_err(|e| format!("Failed to insert default remote control settings: {}", e))?;

        Ok(Self { conn })
    }

    pub fn new() -> Result<Self, String> {
        let data_dir = dirs::data_dir()
            .ok_or("Could not determine data directory")?
            .join("qbz");
        Self::open_at(&data_dir, "remote_control_settings.db")
    }

    pub fn new_at(base_dir: &Path) -> Result<Self, String> {
        Self::open_at(base_dir, "remote_control_settings.db")
    }

    pub fn get_settings(&self) -> Result<RemoteControlSettings, String> {
        let mut settings = self
            .conn
            .query_row(
                "SELECT enabled, port, secure, token FROM remote_control_settings WHERE id = 1",
                [],
                |row| {
                    let enabled: i32 = row.get(0)?;
                    let port: i64 = row.get(1)?;
                    let secure: i32 = row.get(2)?;
                    let token: String = row.get(3)?;
                    Ok(RemoteControlSettings {
                        enabled: enabled != 0,
                        port: port as u16,
                        secure: secure != 0,
                        token,
                    })
                },
            )
            .map_err(|e| format!("Failed to get remote control settings: {}", e))?;

        if settings.token.is_empty() {
            settings.token = generate_token();
            self.set_token(&settings.token)?;
        }

        Ok(settings)
    }
}

pub(super) fn generate_token() -> String {
    let mut bytes = [0u8; 32];
    rand::rng().fill(&mut bytes);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

fn ensure_secure_column(conn: &Connection) -> Result<(), String> {
    let mut stmt = conn
        .prepare("PRAGMA table_info(remote_control_settings)")
        .map_err(|e| format!("Failed to read settings schema: {}", e))?;
    let mut rows = stmt
        .query([])
        .map_err(|e| format!("Failed to read settings schema: {}", e))?;

    while let Some(row) = rows
        .next()
        .map_err(|e| format!("Schema read error: {}", e))?
    {
        let name: String = row
            .get(1)
            .map_err(|e| format!("Schema read error: {}", e))?;
        if name == "secure" {
            return Ok(());
        }
    }

    conn.execute(
        "ALTER TABLE remote_control_settings ADD COLUMN secure INTEGER NOT NULL DEFAULT 0",
        [],
    )
    .map_err(|e| format!("Failed to migrate remote control settings: {}", e))?;

    Ok(())
}
