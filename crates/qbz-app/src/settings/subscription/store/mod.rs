use rusqlite::Connection;
use std::path::Path;

mod queries;

pub struct SubscriptionStateStore {
    conn: Connection,
}

impl SubscriptionStateStore {
    fn open_at(dir: &Path, db_name: &str) -> Result<Self, String> {
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("Failed to create data directory: {}", e))?;

        let db_path = dir.join(db_name);
        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open subscription state database: {}", e))?;

        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
            .map_err(|e| {
                format!(
                    "Failed to enable WAL for subscription state database: {}",
                    e
                )
            })?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS subscription_state (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                invalid_since INTEGER,
                last_invalid_at INTEGER,
                last_valid_at INTEGER,
                last_checked_at INTEGER,
                downloads_purged_at INTEGER
            );
            INSERT OR IGNORE INTO subscription_state (id) VALUES (1);",
        )
        .map_err(|e| format!("Failed to create subscription state table: {}", e))?;

        Ok(Self { conn })
    }

    pub fn new() -> Result<Self, String> {
        let data_dir = dirs::data_dir()
            .ok_or("Could not determine data directory")?
            .join("qbz");
        Self::open_at(&data_dir, "subscription_state.db")
    }

    pub fn new_at(base_dir: &Path) -> Result<Self, String> {
        Self::open_at(base_dir, "subscription_state.db")
    }
}
