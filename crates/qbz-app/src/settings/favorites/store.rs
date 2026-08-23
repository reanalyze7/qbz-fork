use chrono::Utc;
use md5::{Digest, Md5};
use rusqlite::{Connection, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub struct FavoritesPreferencesStore {
    pub(super) conn: Connection,
}

impl FavoritesPreferencesStore {
    fn open_at(dir: &Path, db_name: &str) -> Result<Self, String> {
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("Failed to create data directory: {}", e))?;

        let db_path = dir.join(db_name);
        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open favorites preferences database: {}", e))?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
            .map_err(|e| format!("Failed to set WAL mode on favorites preferences: {}", e))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS favorites_preferences (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                custom_icon_path TEXT,
                custom_icon_preset TEXT,
                tab_order TEXT NOT NULL
            )",
            [],
        )
        .map_err(|e| format!("Failed to create favorites preferences table: {}", e))?;

        // Migration: Add icon_background column if it doesn't exist
        let has_icon_background = conn
            .prepare("SELECT icon_background FROM favorites_preferences LIMIT 1")
            .is_ok();

        if !has_icon_background {
            conn.execute(
                "ALTER TABLE favorites_preferences ADD COLUMN icon_background TEXT",
                [],
            )
            .map_err(|e| format!("Failed to add icon_background column: {}", e))?;
        }

        Ok(Self { conn })
    }

    pub fn new() -> Result<Self, String> {
        let data_dir = dirs::data_dir()
            .ok_or("Could not determine data directory")?
            .join("qbz");
        Self::open_at(&data_dir, "favorites_preferences.db")
    }

    pub fn new_at(base_dir: &Path) -> Result<Self, String> {
        Self::open_at(base_dir, "favorites_preferences.db")
    }

    pub(super) fn favorites_icon_dir() -> Result<PathBuf, String> {
        let cache_dir = dirs::cache_dir()
            .ok_or("Could not determine cache directory")?
            .join("qbz")
            .join("favorites");

        fs::create_dir_all(&cache_dir)
            .map_err(|e| format!("Failed to create favorites icon directory: {}", e))?;

        Ok(cache_dir)
    }

    pub(super) fn normalize_custom_icon_path(&self, path: String) -> Result<String, String> {
        let trimmed = path.trim();
        if trimmed.is_empty() {
            return Ok(String::new());
        }

        let source = Path::new(trimmed);
        if !source.exists() {
            return Err(format!("Source icon does not exist: {}", trimmed));
        }

        let icon_dir = Self::favorites_icon_dir()?;
        if source.starts_with(&icon_dir) {
            return Ok(trimmed.to_string());
        }

        let extension = source.extension().and_then(|e| e.to_str()).unwrap_or("png");

        let mut hasher = Md5::new();
        hasher.update(trimmed.as_bytes());
        let hash = format!("{:x}", hasher.finalize());
        let filename = format!(
            "favorites_icon_{}_{}.{}",
            hash,
            Utc::now().timestamp(),
            extension
        );
        let dest_path = icon_dir.join(filename);

        fs::copy(source, &dest_path)
            .map_err(|e| format!("Failed to copy favorites icon: {}", e))?;

        Ok(dest_path.to_string_lossy().to_string())
    }
}
