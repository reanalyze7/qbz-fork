//! Store initialization: opening the DB, schema DDL, and loading the
//! in-memory artist index from disk.

use super::ArtistVectorStore;
use rusqlite::Connection;
use std::collections::HashMap;
use std::path::Path;

impl ArtistVectorStore {
    /// Open the per-user store at `<base_dir>/cache/artist_vectors.db` (WAL),
    /// creating the schema + loading the artist index. Mirrors Tauri's
    /// `ArtistVectorStoreState::init_at` + `ArtistVectorStore::new`.
    pub fn open_at(base_dir: &Path) -> Result<Self, String> {
        let cache_dir = base_dir.join("cache");
        std::fs::create_dir_all(&cache_dir)
            .map_err(|e| format!("Failed to create cache directory: {}", e))?;
        let db_path = cache_dir.join("artist_vectors.db");

        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open artist vector store: {}", e))?;

        // Enable WAL mode for better concurrency
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
            .map_err(|e| format!("Failed to set pragmas: {}", e))?;

        let mut store = Self {
            conn,
            artist_to_idx: HashMap::new(),
            idx_to_artist: Vec::new(),
            next_idx: 0,
        };

        store.init()?;
        store.load_artist_index()?;

        log::info!("Artist vector store initialized at {:?}", db_path);
        Ok(store)
    }

    /// Initialize database schema (kept byte-identical with Tauri for reuse).
    fn init(&self) -> Result<(), String> {
        self.conn
            .execute_batch(
                r#"
                -- Artist index: maps MBID to integer index for vectors
                CREATE TABLE IF NOT EXISTS artist_index (
                    idx INTEGER PRIMARY KEY,
                    mbid TEXT UNIQUE NOT NULL,
                    name TEXT,
                    created_at INTEGER NOT NULL DEFAULT (unixepoch())
                );
                CREATE INDEX IF NOT EXISTS idx_artist_index_mbid ON artist_index(mbid);

                -- Vector entries: sparse representation (one row per non-zero)
                CREATE TABLE IF NOT EXISTS vector_entries (
                    artist_idx INTEGER NOT NULL,
                    target_idx INTEGER NOT NULL,
                    weight REAL NOT NULL,
                    source TEXT NOT NULL,
                    updated_at INTEGER NOT NULL DEFAULT (unixepoch()),
                    PRIMARY KEY (artist_idx, target_idx, source)
                );
                CREATE INDEX IF NOT EXISTS idx_vector_entries_artist ON vector_entries(artist_idx);
                CREATE INDEX IF NOT EXISTS idx_vector_entries_target ON vector_entries(target_idx);
                CREATE INDEX IF NOT EXISTS idx_vector_entries_updated ON vector_entries(updated_at);

                -- Vector metadata: track when each artist's vector was last computed
                CREATE TABLE IF NOT EXISTS vector_metadata (
                    artist_idx INTEGER PRIMARY KEY,
                    updated_at INTEGER NOT NULL,
                    nnz INTEGER NOT NULL DEFAULT 0
                );
                "#,
            )
            .map_err(|e| format!("Failed to initialize schema: {}", e))?;

        Ok(())
    }

    /// Load artist index from database into memory
    fn load_artist_index(&mut self) -> Result<(), String> {
        let mut stmt = self
            .conn
            .prepare("SELECT idx, mbid FROM artist_index ORDER BY idx")
            .map_err(|e| format!("Failed to prepare index query: {}", e))?;

        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, u32>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| format!("Failed to query index: {}", e))?;

        self.artist_to_idx.clear();
        self.idx_to_artist.clear();

        for row in rows {
            let (idx, mbid) = row.map_err(|e| format!("Failed to read row: {}", e))?;
            self.artist_to_idx.insert(mbid.clone(), idx);

            // Ensure idx_to_artist has enough capacity
            while self.idx_to_artist.len() <= idx as usize {
                self.idx_to_artist.push(String::new());
            }
            self.idx_to_artist[idx as usize] = mbid;

            if idx >= self.next_idx {
                self.next_idx = idx + 1;
            }
        }

        log::debug!(
            "Loaded {} artists into index, next_idx={}",
            self.artist_to_idx.len(),
            self.next_idx
        );

        Ok(())
    }
}
