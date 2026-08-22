//! `AudioSettingsStore` struct and its constructors. The actual table
//! creation/migrations live in `schema.rs` and `seed.rs`; getters live in
//! `store_get.rs`; setters are split by concern into `store_setters_*.rs`.

use rusqlite::Connection;
use std::path::Path;

pub struct AudioSettingsStore {
    pub(crate) conn: Connection,
}

impl AudioSettingsStore {
    fn open_at(dir: &Path, db_name: &str) -> Result<Self, String> {
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("Failed to create data directory: {}", e))?;

        let db_path = dir.join(db_name);
        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open audio settings database: {}", e))?;

        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
            .map_err(|e| format!("Failed to enable WAL for audio settings database: {}", e))?;

        super::schema::create_and_migrate_schema(&conn)?;
        super::seed::seed_default_row(&conn)?;
        super::seed::backfill_legacy_defaults(&conn)?;

        Ok(Self { conn })
    }

    pub fn new() -> Result<Self, String> {
        let data_dir = dirs::data_dir()
            .ok_or("Could not determine data directory")?
            .join("qbz");
        Self::open_at(&data_dir, "audio_settings.db")
    }

    pub fn new_at(base_dir: &Path) -> Result<Self, String> {
        Self::open_at(base_dir, "audio_settings.db")
    }
}
