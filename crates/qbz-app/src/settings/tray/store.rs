use super::prefs::{normalize_tray_icon_theme, TraySettings};
use log::info;
use rusqlite::Connection;
use std::path::Path;

pub struct TraySettingsStore {
    pub(super) conn: Connection,
}

impl TraySettingsStore {
    fn open_at(dir: &Path, db_name: &str) -> Result<Self, String> {
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("Failed to create data directory: {}", e))?;

        let db_path = dir.join(db_name);
        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open tray settings database: {}", e))?;

        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
            .map_err(|e| format!("Failed to enable WAL for tray settings database: {}", e))?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS tray_settings (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                enable_tray INTEGER NOT NULL DEFAULT 1,
                minimize_to_tray INTEGER NOT NULL DEFAULT 0,
                close_to_tray INTEGER NOT NULL DEFAULT 1
            );",
        )
        .map_err(|e| format!("Failed to create tray settings table: {}", e))?;

        let has_theme_column: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('tray_settings') WHERE name = 'tray_icon_theme'",
                [],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to check tray_icon_theme column: {}", e))?;
        if has_theme_column == 0 {
            conn.execute_batch(
                "ALTER TABLE tray_settings ADD COLUMN tray_icon_theme TEXT NOT NULL DEFAULT 'auto';",
            )
            .map_err(|e| format!("Failed to add tray_icon_theme column: {}", e))?;
        }

        let has_mac_hide_dock_column: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('tray_settings') WHERE name = 'mac_hide_dock'",
                [],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to check mac_hide_dock column: {}", e))?;
        if has_mac_hide_dock_column == 0 {
            conn.execute_batch(
                "ALTER TABLE tray_settings ADD COLUMN mac_hide_dock INTEGER NOT NULL DEFAULT 0;",
            )
            .map_err(|e| format!("Failed to add mac_hide_dock column: {}", e))?;
        }

        conn.execute(
            "INSERT OR IGNORE INTO tray_settings (id, enable_tray, minimize_to_tray, close_to_tray, tray_icon_theme, mac_hide_dock)
            VALUES (1, 1, 0, 1, 'auto', 0)",
            [],
        )
        .map_err(|e| format!("Failed to insert default tray settings: {}", e))?;

        info!("[TraySettings] Database initialized");

        Ok(Self { conn })
    }

    pub fn new() -> Result<Self, String> {
        let data_dir = dirs::data_dir()
            .ok_or("Could not determine data directory")?
            .join("qbz");
        Self::open_at(&data_dir, "tray_settings.db")
    }

    pub fn new_at(base_dir: &Path) -> Result<Self, String> {
        Self::open_at(base_dir, "tray_settings.db")
    }

    pub fn get_settings(&self) -> Result<TraySettings, String> {
        self.conn
            .query_row(
                "SELECT enable_tray, minimize_to_tray, close_to_tray, tray_icon_theme, mac_hide_dock FROM tray_settings WHERE id = 1",
                [],
                |row| {
                    let enable_tray: i32 = row.get(0)?;
                    let minimize_to_tray: i32 = row.get(1)?;
                    let close_to_tray: i32 = row.get(2)?;
                    let tray_icon_theme: String = row.get(3)?;
                    let mac_hide_dock: i32 = row.get(4)?;
                    Ok(TraySettings {
                        enable_tray: enable_tray != 0,
                        minimize_to_tray: minimize_to_tray != 0,
                        close_to_tray: close_to_tray != 0,
                        tray_icon_theme: normalize_tray_icon_theme(&tray_icon_theme),
                        mac_hide_dock: mac_hide_dock != 0,
                    })
                },
            )
            .map_err(|e| format!("Failed to get tray settings: {}", e))
    }
}
