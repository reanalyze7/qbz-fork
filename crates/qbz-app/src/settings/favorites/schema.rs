use super::prefs::FavoritesPreferences;
use rusqlite::{params, Connection, Result};

pub fn create_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS favorites_preferences (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            custom_icon_path TEXT,
            custom_icon_preset TEXT,
            tab_order TEXT NOT NULL
        )",
        [],
    )?;

    // Migration: Add icon_background column if it doesn't exist
    let has_icon_background = conn
        .prepare("SELECT icon_background FROM favorites_preferences LIMIT 1")
        .is_ok();

    if !has_icon_background {
        conn.execute(
            "ALTER TABLE favorites_preferences ADD COLUMN icon_background TEXT",
            [],
        )?;
    }

    Ok(())
}

pub fn load_preferences(conn: &Connection) -> Result<FavoritesPreferences> {
    let mut stmt = conn.prepare("SELECT custom_icon_path, custom_icon_preset, icon_background, tab_order FROM favorites_preferences WHERE id = 1")?;

    let result = stmt.query_row([], |row| {
        let custom_icon_path: Option<String> = row.get(0)?;
        let custom_icon_preset: Option<String> = row.get(1)?;
        let icon_background: Option<String> = row.get(2)?;
        let tab_order_str: String = row.get(3)?;

        let custom_icon_path = custom_icon_path.and_then(|value| {
            if value.trim().is_empty() {
                None
            } else {
                Some(value)
            }
        });

        let tab_order: Vec<String> = serde_json::from_str(&tab_order_str).unwrap_or_else(|_| {
            vec![
                "tracks".to_string(),
                "albums".to_string(),
                "artists".to_string(),
                "playlists".to_string(),
            ]
        });

        Ok(FavoritesPreferences {
            custom_icon_path,
            custom_icon_preset,
            icon_background,
            tab_order,
        })
    });

    match result {
        Ok(prefs) => Ok(prefs),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(FavoritesPreferences::default()),
        Err(e) => Err(e),
    }
}

pub fn save_preferences(conn: &Connection, prefs: &FavoritesPreferences) -> Result<()> {
    let tab_order_str = serde_json::to_string(&prefs.tab_order).unwrap();

    conn.execute(
        "INSERT OR REPLACE INTO favorites_preferences (id, custom_icon_path, custom_icon_preset, icon_background, tab_order)
         VALUES (1, ?1, ?2, ?3, ?4)",
        params![prefs.custom_icon_path, prefs.custom_icon_preset, prefs.icon_background, tab_order_str],
    )?;
    Ok(())
}
