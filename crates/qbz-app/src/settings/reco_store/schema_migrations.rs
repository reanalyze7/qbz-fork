use rusqlite::Connection;

/// Idempotent: add `genre_id` (+ its index) if an old Tauri DB lacks it.
pub(super) fn migrate_add_genre_id(conn: &Connection) -> Result<(), String> {
    let has_column: bool = conn
        .prepare("PRAGMA table_info(reco_events)")
        .map_err(|e| format!("Failed to query table info: {}", e))?
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| format!("Failed to read table info: {}", e))?
        .filter_map(Result::ok)
        .any(|col| col == "genre_id");

    if !has_column {
        conn.execute("ALTER TABLE reco_events ADD COLUMN genre_id INTEGER", [])
            .map_err(|e| format!("Failed to add genre_id column: {}", e))?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_reco_events_genre ON reco_events(genre_id)",
            [],
        )
        .map_err(|e| format!("Failed to create genre_id index: {}", e))?;
    }
    Ok(())
}
