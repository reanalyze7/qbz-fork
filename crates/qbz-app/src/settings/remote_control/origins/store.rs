use rusqlite::{params, Connection};
use std::path::Path;

use super::AllowedOrigin;
use crate::settings::remote_control::DEFAULT_ALLOWED_ORIGINS;

pub struct AllowedOriginsStore {
    pub(super) conn: Connection,
}

impl AllowedOriginsStore {
    fn open_at(dir: &Path, db_name: &str) -> Result<Self, String> {
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("Failed to create data directory: {}", e))?;

        let db_path = dir.join(db_name);
        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open allowed origins database: {}", e))?;

        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
            .map_err(|e| format!("Failed to enable WAL for allowed origins database: {}", e))?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS allowed_origins (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                origin TEXT NOT NULL UNIQUE,
                is_default INTEGER NOT NULL DEFAULT 0
            );",
        )
        .map_err(|e| format!("Failed to create allowed_origins table: {}", e))?;

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM allowed_origins", [], |row| row.get(0))
            .unwrap_or(0);

        if count == 0 {
            for origin in DEFAULT_ALLOWED_ORIGINS {
                let _ = conn.execute(
                    "INSERT OR IGNORE INTO allowed_origins (origin, is_default) VALUES (?1, 1)",
                    params![origin],
                );
            }
        }

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

    pub fn get_origins(&self) -> Result<Vec<AllowedOrigin>, String> {
        let mut stmt = self.conn
            .prepare("SELECT id, origin, is_default FROM allowed_origins ORDER BY is_default DESC, origin ASC")
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let origins = stmt
            .query_map([], |row| {
                Ok(AllowedOrigin {
                    id: row.get(0)?,
                    origin: row.get(1)?,
                    is_default: row.get::<_, i32>(2)? != 0,
                })
            })
            .map_err(|e| format!("Failed to query origins: {}", e))?;

        origins
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect origins: {}", e))
    }

    pub fn is_origin_allowed(&self, origin: &str) -> bool {
        self.conn
            .query_row(
                "SELECT 1 FROM allowed_origins WHERE origin = ?1",
                params![origin],
                |_| Ok(()),
            )
            .is_ok()
    }
}
