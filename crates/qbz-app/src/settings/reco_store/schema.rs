use rusqlite::Connection;
use std::path::Path;

pub struct RecoStore {
    pub(super) conn: Connection,
}

impl RecoStore {
    fn open_at(reco_dir: &Path) -> Result<Self, String> {
        std::fs::create_dir_all(reco_dir)
            .map_err(|e| format!("Failed to create reco directory: {}", e))?;

        // Same filename as Tauri (src-tauri/src/reco_store/mod.rs:166): events.db
        let db_path = reco_dir.join("events.db");
        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open reco database: {}", e))?;

        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
            .map_err(|e| format!("Failed to enable WAL for reco database: {}", e))?;

        let store = Self { conn };
        store.init()?;
        Ok(store)
    }

    /// Default (non per-user) location: `dirs::data_dir()/qbz/reco/events.db`,
    /// matching Tauri's `RecoState::new` (mod.rs:140-148).
    pub fn new() -> Result<Self, String> {
        let reco_dir = dirs::data_dir()
            .ok_or("Could not determine data directory")?
            .join("qbz")
            .join("reco");
        Self::open_at(&reco_dir)
    }

    /// Per-user location: `<base_dir>/reco/events.db`, matching Tauri's
    /// `RecoState::init_at` (mod.rs:162-167). Shares the user's existing
    /// Tauri reco history.
    pub fn new_at(base_dir: &Path) -> Result<Self, String> {
        Self::open_at(&base_dir.join("reco"))
    }

    /// Idempotent schema creation. Matches Tauri's `RecoStoreDb::init` exactly,
    /// except `genre_id` is included inline in the base `reco_events` table (the
    /// migration is still run so an OLD Tauri DB without the column is upgraded).
    fn init(&self) -> Result<(), String> {
        self.conn
            .execute_batch(
                r#"
                CREATE TABLE IF NOT EXISTS reco_events (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    event_type TEXT NOT NULL,
                    item_type TEXT NOT NULL,
                    track_id INTEGER,
                    album_id TEXT,
                    artist_id INTEGER,
                    playlist_id INTEGER,
                    genre_id INTEGER,
                    created_at INTEGER NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_reco_events_type ON reco_events(event_type);
                CREATE INDEX IF NOT EXISTS idx_reco_events_track ON reco_events(track_id);
                CREATE INDEX IF NOT EXISTS idx_reco_events_album ON reco_events(album_id);
                CREATE INDEX IF NOT EXISTS idx_reco_events_artist ON reco_events(artist_id);
                CREATE INDEX IF NOT EXISTS idx_reco_events_created ON reco_events(created_at);
                CREATE INDEX IF NOT EXISTS idx_reco_events_genre ON reco_events(genre_id);

                CREATE INDEX IF NOT EXISTS idx_reco_events_play_albums
                    ON reco_events(event_type, album_id, created_at DESC)
                    WHERE album_id IS NOT NULL;
                CREATE INDEX IF NOT EXISTS idx_reco_events_play_tracks
                    ON reco_events(event_type, track_id, created_at DESC)
                    WHERE track_id IS NOT NULL;
                CREATE INDEX IF NOT EXISTS idx_reco_events_play_artists
                    ON reco_events(event_type, artist_id, created_at DESC)
                    WHERE artist_id IS NOT NULL;

                CREATE TABLE IF NOT EXISTS reco_scores (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    score_type TEXT NOT NULL,
                    item_type TEXT NOT NULL,
                    track_id INTEGER,
                    album_id TEXT,
                    artist_id INTEGER,
                    score REAL NOT NULL,
                    updated_at INTEGER NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_reco_scores_type ON reco_scores(score_type);
                CREATE INDEX IF NOT EXISTS idx_reco_scores_item ON reco_scores(item_type);
                CREATE INDEX IF NOT EXISTS idx_reco_scores_track ON reco_scores(track_id);
                CREATE INDEX IF NOT EXISTS idx_reco_scores_album ON reco_scores(album_id);
                CREATE INDEX IF NOT EXISTS idx_reco_scores_artist ON reco_scores(artist_id);
                CREATE INDEX IF NOT EXISTS idx_reco_scores_lookup
                    ON reco_scores(score_type, item_type, score DESC);

                CREATE TABLE IF NOT EXISTS reco_album_meta (
                    album_id TEXT PRIMARY KEY,
                    title TEXT NOT NULL,
                    artist_name TEXT NOT NULL,
                    artist_id INTEGER,
                    artwork_url TEXT NOT NULL DEFAULT '',
                    genre_name TEXT NOT NULL DEFAULT '',
                    quality TEXT NOT NULL DEFAULT '',
                    release_date TEXT,
                    updated_at INTEGER NOT NULL
                );
                "#,
            )
            .map_err(|e| format!("Failed to initialize reco database: {}", e))?;

        // Upgrade an old Tauri DB whose base schema predates the genre_id column.
        super::schema_migrations::migrate_add_genre_id(&self.conn)?;

        Ok(())
    }
}
