//! Local cache for favorite track / album / artist / label IDs.
//!
//! Hoisted from `src-tauri/src/config/favorites_cache.rs` (frontend-agnostic,
//! ADR-006) so non-Tauri frontends can read favorite status offline. The db
//! filename and schema are kept IDENTICAL to the Tauri store — both frontends
//! open the same per-user `favorites_cache.db`.
//!
//! Sync strategy (mirrors Tauri):
//! - On login: fetch all favorites from the API and replace the cache
//! - On toggle: API call first, then update the local cache on success

mod albums;
mod artists;
mod labels;
#[cfg(test)]
mod tests;
mod tracks;

use rusqlite::Connection;
use std::path::Path;

pub struct FavoritesCacheStore {
    conn: Connection,
}

impl FavoritesCacheStore {
    fn open_at(dir: &Path, db_name: &str) -> Result<Self, String> {
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("Failed to create data directory: {}", e))?;

        let db_path = dir.join(db_name);
        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open favorites cache database: {}", e))?;

        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
            .map_err(|e| format!("Failed to enable WAL for favorites cache database: {}", e))?;

        // Create tables
        conn.execute(
            "CREATE TABLE IF NOT EXISTS favorite_tracks (
                track_id INTEGER PRIMARY KEY,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )
        .map_err(|e| format!("Failed to create favorite_tracks table: {}", e))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS favorite_albums (
                album_id TEXT PRIMARY KEY,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )
        .map_err(|e| format!("Failed to create favorite_albums table: {}", e))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS favorite_artists (
                artist_id INTEGER PRIMARY KEY,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )
        .map_err(|e| format!("Failed to create favorite_artists table: {}", e))?;

        // Labels — added as part of the Follow Label feature; same shape as
        // favorite_artists. CREATE IF NOT EXISTS is the migration story
        // for existing databases (no separate ALTER needed).
        conn.execute(
            "CREATE TABLE IF NOT EXISTS favorite_labels (
                label_id INTEGER PRIMARY KEY,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )
        .map_err(|e| format!("Failed to create favorite_labels table: {}", e))?;

        Ok(Self { conn })
    }

    pub fn new() -> Result<Self, String> {
        let data_dir = dirs::data_dir()
            .ok_or("Could not determine data directory")?
            .join("qbz");
        Self::open_at(&data_dir, "favorites_cache.db")
    }

    pub fn new_at(base_dir: &Path) -> Result<Self, String> {
        Self::open_at(base_dir, "favorites_cache.db")
    }

    // ============ Clear all (for logout) ============

    pub fn clear_all(&self) -> Result<(), String> {
        self.conn
            .execute("DELETE FROM favorite_tracks", [])
            .map_err(|e| format!("Failed to clear favorite tracks: {}", e))?;
        self.conn
            .execute("DELETE FROM favorite_albums", [])
            .map_err(|e| format!("Failed to clear favorite albums: {}", e))?;
        self.conn
            .execute("DELETE FROM favorite_artists", [])
            .map_err(|e| format!("Failed to clear favorite artists: {}", e))?;
        self.conn
            .execute("DELETE FROM favorite_labels", [])
            .map_err(|e| format!("Failed to clear favorite labels: {}", e))?;
        Ok(())
    }
}
