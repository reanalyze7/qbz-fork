use rusqlite::Connection;
use std::collections::HashSet;
use std::path::Path;
use std::sync::RwLock;

/// Local-favorites service with O(1) lookup performance.
pub struct LocalFavoritesService {
    pub(super) conn: Connection,
    /// In-memory `(kind, id)` set for O(1) heart lookups.
    pub(super) keys: RwLock<HashSet<(String, String)>>,
}

impl LocalFavoritesService {
    /// Create a new service, opening or creating the database.
    pub fn new(db_path: &Path) -> Result<Self, String> {
        log::info!("[LocalFav] Opening database at: {}", db_path.display());

        let conn = Connection::open(db_path)
            .map_err(|e| format!("Failed to open local favorites database: {}", e))?;

        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
            .map_err(|e| format!("Failed to set WAL mode: {}", e))?;

        let service = Self {
            conn,
            keys: RwLock::new(HashSet::new()),
        };

        service.init_schema()?;
        service.load_from_db()?;

        Ok(service)
    }

    /// Create an in-memory service (test/ephemeral helper).
    pub fn new_in_memory() -> Result<Self, String> {
        let conn = Connection::open_in_memory()
            .map_err(|e| format!("Failed to open in-memory local favorites database: {}", e))?;

        let service = Self {
            conn,
            keys: RwLock::new(HashSet::new()),
        };

        service.init_schema()?;
        service.load_from_db()?;

        Ok(service)
    }

    fn init_schema(&self) -> Result<(), String> {
        self.conn
            .execute_batch(
                r#"
                CREATE TABLE IF NOT EXISTS local_favorites (
                    kind TEXT NOT NULL CHECK (kind IN ('album','artist','track')),
                    id TEXT NOT NULL,
                    title TEXT NOT NULL,
                    subtitle TEXT,
                    artwork_url TEXT,
                    artist TEXT,
                    source TEXT NOT NULL CHECK (source IN ('local')),
                    favorited_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
                    PRIMARY KEY (kind, id)
                );
                CREATE INDEX IF NOT EXISTS idx_local_favorites_at
                    ON local_favorites(favorited_at);
                CREATE INDEX IF NOT EXISTS idx_local_favorites_artist
                    ON local_favorites(kind, artist);
                "#,
            )
            .map_err(|e| format!("Failed to initialize local favorites schema: {}", e))?;

        Ok(())
    }

    fn load_from_db(&self) -> Result<(), String> {
        let mut stmt = self
            .conn
            .prepare("SELECT kind, id FROM local_favorites")
            .map_err(|e| format!("Failed to prepare local favorites query: {}", e))?;

        let keys: Vec<(String, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| format!("Failed to query local favorites: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        let count = keys.len();
        let mut set = self
            .keys
            .write()
            .map_err(|_| "Failed to acquire write lock")?;
        *set = keys.into_iter().collect();

        log::info!("[LocalFav] Loaded {} local favorites into memory", count);
        Ok(())
    }
}
