use super::settings::GraphicsSettings;
use rusqlite::Connection;
use std::path::Path;

pub struct GraphicsSettingsStore {
    pub(super) conn: Connection,
}

impl GraphicsSettingsStore {
    /// Lightweight read-only open for startup before host-managed state exists.
    /// Opens existing DB without creating tables or running migrations.
    pub fn new_readonly() -> Result<Self, String> {
        let db_path = dirs::data_dir()
            .ok_or("Could not determine data directory")?
            .join("qbz")
            .join("graphics_settings.db");
        Self::new_readonly_at_path(&db_path)
    }

    pub fn new_readonly_at_path(db_path: &Path) -> Result<Self, String> {
        if !db_path.exists() {
            return Err("Graphics settings DB does not exist yet".to_string());
        }

        let conn = Connection::open_with_flags(
            db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|e| {
            format!(
                "Failed to open graphics settings database (readonly): {}",
                e
            )
        })?;

        Ok(Self { conn })
    }

    fn open_at(dir: &Path, db_name: &str) -> Result<Self, String> {
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("Failed to create data directory: {}", e))?;

        let db_path = dir.join(db_name);
        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open graphics settings database: {}", e))?;

        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
            .map_err(|e| format!("Failed to enable WAL for graphics settings database: {}", e))?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS graphics_settings (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                hardware_acceleration INTEGER NOT NULL DEFAULT 1
            );
            INSERT OR IGNORE INTO graphics_settings (id, hardware_acceleration) VALUES (1, 1);",
        )
        .map_err(|e| format!("Failed to create graphics settings table: {}", e))?;

        let _ = conn.execute_batch(
            "ALTER TABLE graphics_settings ADD COLUMN force_x11 INTEGER NOT NULL DEFAULT 0;",
        );
        let _ = conn.execute_batch("ALTER TABLE graphics_settings ADD COLUMN gdk_scale TEXT;");
        let _ = conn.execute_batch("ALTER TABLE graphics_settings ADD COLUMN gdk_dpi_scale TEXT;");
        let _ = conn.execute_batch("ALTER TABLE graphics_settings ADD COLUMN gsk_renderer TEXT;");
        let _ = conn.execute_batch(
            "ALTER TABLE graphics_settings ADD COLUMN preferred_gpu TEXT NOT NULL DEFAULT 'auto';",
        );
        let _ = conn.execute_batch(
            "ALTER TABLE graphics_settings ADD COLUMN nvidia_compat_mode INTEGER NOT NULL DEFAULT 0;",
        );

        Ok(Self { conn })
    }

    pub fn new() -> Result<Self, String> {
        let data_dir = dirs::data_dir()
            .ok_or("Could not determine data directory")?
            .join("qbz");
        Self::open_at(&data_dir, "graphics_settings.db")
    }

    pub fn new_at(base_dir: &Path) -> Result<Self, String> {
        Self::open_at(base_dir, "graphics_settings.db")
    }

    pub fn get_settings(&self) -> Result<GraphicsSettings, String> {
        self.conn
            .query_row(
                "SELECT hardware_acceleration, force_x11, gdk_scale, gdk_dpi_scale, gsk_renderer, preferred_gpu, nvidia_compat_mode FROM graphics_settings WHERE id = 1",
                [],
                |row| {
                    Ok(GraphicsSettings {
                        hardware_acceleration: row.get::<_, i64>(0)? != 0,
                        force_x11: row.get::<_, i64>(1)? != 0,
                        gdk_scale: row.get::<_, Option<String>>(2)?,
                        gdk_dpi_scale: row.get::<_, Option<String>>(3)?,
                        gsk_renderer: row.get::<_, Option<String>>(4)?,
                        preferred_gpu: row
                            .get::<_, Option<String>>(5)?
                            .unwrap_or_else(|| "auto".to_string()),
                        nvidia_compat_mode: row.get::<_, i64>(6).unwrap_or(0) != 0,
                    })
                },
            )
            .map_err(|e| format!("Failed to get graphics settings: {}", e))
    }
}
