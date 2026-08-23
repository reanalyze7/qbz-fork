use rusqlite::{params, Connection};
use serde_json::Value;
use std::path::Path;
use std::sync::{Arc, Mutex};

use super::defaults::default_prefs;
use super::model::DiscoverPrefs;

pub struct DiscoverPrefsStore {
    pub(super) conn: Connection,
}

impl DiscoverPrefsStore {
    fn open_at(dir: &Path, db_name: &str) -> Result<Self, String> {
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("Failed to create data directory: {}", e))?;

        let db_path = dir.join(db_name);
        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open discover prefs database: {}", e))?;

        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
            .map_err(|e| format!("Failed to enable WAL for discover prefs database: {}", e))?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS discover_prefs (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                prefs_json TEXT NOT NULL
            );",
        )
        .map_err(|e| format!("Failed to create discover prefs table: {}", e))?;

        conn.execute(
            "INSERT OR IGNORE INTO discover_prefs (id, prefs_json) VALUES (1, ?1)",
            params![default_prefs().to_json().to_string()],
        )
        .map_err(|e| format!("Failed to initialize discover prefs: {}", e))?;

        Ok(Self { conn })
    }

    pub fn new() -> Result<Self, String> {
        let data_dir = dirs::data_dir()
            .ok_or("Could not determine data directory")?
            .join("qbz");
        Self::open_at(&data_dir, "discover_prefs.db")
    }

    /// Open the store in a specific (per-user) base directory.
    pub fn new_at(base_dir: &Path) -> Result<Self, String> {
        Self::open_at(base_dir, "discover_prefs.db")
    }

    /// Load and migrate the persisted prefs. A missing / corrupt / unparseable
    /// blob yields defaults (never an error to the caller).
    pub fn load(&self) -> DiscoverPrefs {
        let raw: Result<String, _> = self.conn.query_row(
            "SELECT prefs_json FROM discover_prefs WHERE id = 1",
            [],
            |row| row.get(0),
        );
        match raw {
            Ok(text) => match serde_json::from_str::<Value>(&text) {
                Ok(value) => DiscoverPrefs::migrate(&value),
                Err(_) => default_prefs(),
            },
            Err(_) => default_prefs(),
        }
    }

    /// Persist the whole prefs blob (upsert row 1).
    pub fn save(&self, prefs: &DiscoverPrefs) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT INTO discover_prefs (id, prefs_json) VALUES (1, ?1)
                 ON CONFLICT(id) DO UPDATE SET prefs_json = excluded.prefs_json",
                params![prefs.to_json().to_string()],
            )
            .map_err(|e| format!("Failed to save discover prefs: {}", e))?;
        Ok(())
    }
}

pub type DiscoverPrefsState = Arc<Mutex<Option<DiscoverPrefsStore>>>;

pub fn create_empty_discover_prefs_state() -> DiscoverPrefsState {
    Arc::new(Mutex::new(None))
}
