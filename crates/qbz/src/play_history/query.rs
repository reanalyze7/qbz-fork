//! `known_artists`: threshold query over the play-history store.

use std::collections::HashSet;

use rusqlite::params;

use qbz_core::normalize_artist_name;

use super::db::with_db;

/// Known artists with strictly more than `threshold` plays. Returns
/// the Qobuz id set (for filtering validated Qobuz results) and the
/// normalized-name set (for filtering MB candidates). Same two-axis
/// filter the Tauri discovery pipeline applies.
#[allow(dead_code)] // wired by artist::load_mb_discovery
pub fn known_artists(threshold: u32) -> (HashSet<u64>, HashSet<String>) {
    let pair = with_db(|conn| {
        let mut stmt = conn
            .prepare(
                r#"
                SELECT a.artist_id, a.name
                FROM artist_names a
                JOIN (
                    SELECT artist_id, COUNT(*) AS play_count
                    FROM play_events
                    GROUP BY artist_id
                    HAVING play_count > ?
                ) p ON p.artist_id = a.artist_id
                "#,
            )
            .ok()?;
        let rows = stmt
            .query_map(params![threshold], |row| -> rusqlite::Result<(u64, String)> {
                let id: i64 = row.get(0)?;
                let name: String = row.get(1)?;
                Ok((id as u64, name))
            })
            .ok()?;
        let mut ids: HashSet<u64> = HashSet::new();
        let mut names: HashSet<String> = HashSet::new();
        for row in rows.flatten() {
            ids.insert(row.0);
            names.insert(normalize_artist_name(&row.1));
        }
        Some((ids, names))
    });
    pair.unwrap_or_default()
}
