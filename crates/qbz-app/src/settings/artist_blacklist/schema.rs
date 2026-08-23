use std::sync::atomic::Ordering;

use super::BlacklistService;

impl BlacklistService {
    /// Initialize database schema.
    pub(super) fn init_schema(&self) -> Result<(), String> {
        self.conn
            .execute_batch(
                r#"
                -- Artist blacklist entries
                CREATE TABLE IF NOT EXISTS artist_blacklist (
                    artist_id INTEGER PRIMARY KEY,
                    artist_name TEXT NOT NULL,
                    added_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
                    notes TEXT
                );

                -- Index for name search in UI
                CREATE INDEX IF NOT EXISTS idx_artist_blacklist_name
                    ON artist_blacklist(artist_name COLLATE NOCASE);

                -- Settings table (single row)
                CREATE TABLE IF NOT EXISTS blacklist_settings (
                    id INTEGER PRIMARY KEY CHECK (id = 1),
                    enabled INTEGER NOT NULL DEFAULT 1
                );

                -- Insert default settings if not present
                INSERT OR IGNORE INTO blacklist_settings (id, enabled) VALUES (1, 1);

                -- Album blacklist entries (parallel String-keyed axis)
                CREATE TABLE IF NOT EXISTS album_blacklist (
                    album_id TEXT PRIMARY KEY,
                    album_title TEXT NOT NULL,
                    artist_name TEXT,
                    cover_url TEXT,
                    added_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
                    notes TEXT
                );

                -- Index for album title search in UI
                CREATE INDEX IF NOT EXISTS idx_album_blacklist_title
                    ON album_blacklist(album_title COLLATE NOCASE);
                "#,
            )
            .map_err(|e| format!("Failed to initialize blacklist schema: {}", e))?;

        Ok(())
    }

    /// Load all blacklisted IDs from database into memory.
    pub(super) fn load_from_db(&self) -> Result<(), String> {
        let mut stmt = self
            .conn
            .prepare("SELECT artist_id FROM artist_blacklist")
            .map_err(|e| format!("Failed to prepare blacklist query: {}", e))?;

        let ids: Vec<u64> = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| format!("Failed to query blacklist: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        let count = ids.len();
        let mut set = self
            .blacklisted_ids
            .write()
            .map_err(|_| "Failed to acquire write lock")?;
        *set = ids.into_iter().collect();

        log::info!(
            "[Blacklist] Loaded {} blacklisted artists into memory",
            count
        );
        Ok(())
    }

    /// Load all blocked album ids from database into memory.
    pub(super) fn load_albums_from_db(&self) -> Result<(), String> {
        let mut stmt = self
            .conn
            .prepare("SELECT album_id FROM album_blacklist")
            .map_err(|e| format!("Failed to prepare album blacklist query: {}", e))?;

        let ids: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| format!("Failed to query album blacklist: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        let count = ids.len();
        let mut set = self
            .blacklisted_album_ids
            .write()
            .map_err(|_| "Failed to acquire album write lock")?;
        *set = ids.into_iter().collect();

        log::info!("[Blacklist] Loaded {} blocked albums into memory", count);
        Ok(())
    }

    /// Load enabled setting from database.
    pub(super) fn load_settings(&self) -> Result<(), String> {
        let enabled: bool = self
            .conn
            .query_row(
                "SELECT enabled FROM blacklist_settings WHERE id = 1",
                [],
                |row| {
                    let val: i32 = row.get(0)?;
                    Ok(val != 0)
                },
            )
            .map_err(|e| format!("Failed to load blacklist settings: {}", e))?;

        self.enabled.store(enabled, Ordering::Relaxed);
        log::info!("[Blacklist] Feature enabled: {}", enabled);
        Ok(())
    }
}
