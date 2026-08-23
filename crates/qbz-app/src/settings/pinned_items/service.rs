use rusqlite::Connection;
use std::collections::HashSet;
use std::path::Path;
use std::sync::RwLock;

/// Pinned-items service with O(1) lookup performance.
pub struct PinnedItemsService {
    pub(super) conn: Connection,
    /// In-memory `(kind, id)` set for O(1) glyph lookups.
    pub(super) pinned_keys: RwLock<HashSet<(String, String)>>,
}

impl PinnedItemsService {
    /// Create a new pinned-items service, opening or creating the database.
    pub fn new(db_path: &Path) -> Result<Self, String> {
        log::info!("[Pinned] Opening database at: {}", db_path.display());

        let conn = Connection::open(db_path)
            .map_err(|e| format!("Failed to open pinned items database: {}", e))?;

        // Enable WAL mode for better concurrent access (ADR-002).
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
            .map_err(|e| format!("Failed to set WAL mode: {}", e))?;

        let service = Self {
            conn,
            pinned_keys: RwLock::new(HashSet::new()),
        };

        service.init_schema()?;
        service.load_from_db()?;

        Ok(service)
    }

    /// Create an in-memory pinned-items service (test/ephemeral helper).
    ///
    /// Opens a `:memory:` connection and runs schema init + load, but does not
    /// set WAL mode (not needed for an in-memory database).
    pub fn new_in_memory() -> Result<Self, String> {
        let conn = Connection::open_in_memory()
            .map_err(|e| format!("Failed to open in-memory pinned items database: {}", e))?;

        let service = Self {
            conn,
            pinned_keys: RwLock::new(HashSet::new()),
        };

        service.init_schema()?;
        service.load_from_db()?;

        Ok(service)
    }

    /// Initialize database schema.
    fn init_schema(&self) -> Result<(), String> {
        self.conn
            .execute_batch(
                r#"
                -- Pinned entries: (kind, id) composite key + display snapshot
                CREATE TABLE IF NOT EXISTS pinned_items (
                    kind TEXT NOT NULL CHECK (kind IN ('album','artist','playlist')),
                    id TEXT NOT NULL,
                    title TEXT NOT NULL,
                    subtitle TEXT,
                    artwork_url TEXT,
                    pinned_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
                    PRIMARY KEY (kind, id)
                );

                -- Index for the newest-first section ordering
                CREATE INDEX IF NOT EXISTS idx_pinned_items_pinned_at
                    ON pinned_items(pinned_at);
                "#,
            )
            .map_err(|e| format!("Failed to initialize pinned items schema: {}", e))?;

        Ok(())
    }

    /// Load all pinned `(kind, id)` keys from database into memory.
    fn load_from_db(&self) -> Result<(), String> {
        let mut stmt = self
            .conn
            .prepare("SELECT kind, id FROM pinned_items")
            .map_err(|e| format!("Failed to prepare pinned items query: {}", e))?;

        let keys: Vec<(String, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| format!("Failed to query pinned items: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        let count = keys.len();
        let mut set = self
            .pinned_keys
            .write()
            .map_err(|_| "Failed to acquire write lock")?;
        *set = keys.into_iter().collect();

        log::info!("[Pinned] Loaded {} pinned items into memory", count);
        Ok(())
    }
}
