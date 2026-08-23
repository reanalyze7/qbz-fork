use std::collections::HashSet;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::RwLock;

use rusqlite::Connection;

use super::BlacklistService;

impl BlacklistService {
    /// Create a new blacklist service, opening or creating the database.
    pub fn new(db_path: &Path) -> Result<Self, String> {
        log::info!("[Blacklist] Opening database at: {}", db_path.display());

        let conn = Connection::open(db_path)
            .map_err(|e| format!("Failed to open blacklist database: {}", e))?;

        // Enable WAL mode for better concurrent access (ADR-002).
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
            .map_err(|e| format!("Failed to set WAL mode: {}", e))?;

        let service = Self {
            conn,
            blacklisted_ids: RwLock::new(HashSet::new()),
            blacklisted_album_ids: RwLock::new(HashSet::new()),
            enabled: AtomicBool::new(true),
        };

        service.init_schema()?;
        service.load_from_db()?;
        service.load_albums_from_db()?;
        service.load_settings()?;

        Ok(service)
    }

    /// Create an in-memory blacklist service (test/ephemeral helper).
    ///
    /// Opens a `:memory:` connection and runs schema init + loads, but does not
    /// set WAL mode (not needed for an in-memory database).
    pub fn new_in_memory() -> Result<Self, String> {
        let conn = Connection::open_in_memory()
            .map_err(|e| format!("Failed to open in-memory blacklist database: {}", e))?;

        let service = Self {
            conn,
            blacklisted_ids: RwLock::new(HashSet::new()),
            blacklisted_album_ids: RwLock::new(HashSet::new()),
            enabled: AtomicBool::new(true),
        };

        service.init_schema()?;
        service.load_from_db()?;
        service.load_albums_from_db()?;
        service.load_settings()?;

        Ok(service)
    }
}
