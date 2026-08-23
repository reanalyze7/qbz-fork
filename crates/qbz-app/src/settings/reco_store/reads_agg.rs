use rusqlite::params;

use super::now_ts;
use super::schema::RecoStore;

impl RecoStore {
    /// Distinct artist ids the user "knows": played strictly more than
    /// `play_threshold` times OR favorited at least once. Backs the discovery
    /// "skip artists I already know" filter with a richer signal (plays +
    /// favorites) than the play-only `play_history` store.
    pub fn get_known_artist_ids(
        &self,
        play_threshold: u32,
    ) -> Result<std::collections::HashSet<u64>, String> {
        let mut stmt = self
            .conn
            .prepare(
                r#"
                SELECT artist_id
                FROM reco_events
                WHERE artist_id IS NOT NULL
                GROUP BY artist_id
                HAVING SUM(CASE WHEN event_type = 'play' THEN 1 ELSE 0 END) > ?
                    OR SUM(CASE WHEN event_type = 'favorite' THEN 1 ELSE 0 END) > 0
                "#,
            )
            .map_err(|e| format!("Failed to prepare known artists query: {}", e))?;
        let rows = stmt
            .query_map(params![play_threshold], |row| row.get::<_, u64>(0))
            .map_err(|e| format!("Failed to query known artists: {}", e))?;
        let mut ids = std::collections::HashSet::new();
        for row in rows {
            ids.insert(row.map_err(|e| format!("Failed to read known artist row: {}", e))?);
        }
        Ok(ids)
    }

    /// The user's most-played genres by event count (mirrors `get_top_genre_ids`).
    /// Returns `(genre_id, genre_name)` — name from `reco_album_meta` (empty if unknown).
    pub fn get_top_genres(&self, limit: u32) -> Result<Vec<(u64, String)>, String> {
        let mut stmt = self
            .conn
            .prepare(
                r#"
                SELECT e.genre_id, COALESCE(m.genre_name, ''), COUNT(*) AS play_count
                FROM reco_events e
                LEFT JOIN reco_album_meta m ON e.album_id = m.album_id
                WHERE e.genre_id IS NOT NULL AND e.genre_id > 0
                GROUP BY e.genre_id
                ORDER BY play_count DESC
                LIMIT ?
                "#,
            )
            .map_err(|e| format!("Failed to prepare top genres query: {}", e))?;
        let rows = stmt
            .query_map(params![limit], |row| {
                Ok((row.get::<_, u64>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| format!("Failed to query top genres: {}", e))?;
        let mut genres = Vec::new();
        for row in rows {
            genres.push(row.map_err(|e| format!("Failed to read genre row: {}", e))?);
        }
        Ok(genres)
    }

    /// Favorite album ids NOT played within `recency_days` ("forgotten
    /// favorites"). Ported from Tauri `reco_store/db.rs:391-426`, adapted to
    /// the integer `created_at` (unix seconds) schema: Tauri's
    /// `datetime('now', …)` string comparison never matched an integer column,
    /// so the recency filter is a numeric cutoff here.
    pub fn get_forgotten_favorite_album_ids(
        &self,
        limit: u32,
        recency_days: u32,
    ) -> Result<Vec<String>, String> {
        let cutoff = now_ts().saturating_sub((recency_days as i64) * 86_400);
        let mut stmt = self
            .conn
            .prepare(
                r#"
                SELECT f.album_id
                FROM reco_events f
                WHERE f.event_type = 'favorite' AND f.album_id IS NOT NULL
                  AND f.album_id NOT IN (
                    SELECT DISTINCT p.album_id
                    FROM reco_events p
                    WHERE p.event_type = 'play'
                      AND p.album_id IS NOT NULL
                      AND p.created_at > ?
                  )
                GROUP BY f.album_id
                ORDER BY RANDOM()
                LIMIT ?
                "#,
            )
            .map_err(|e| format!("Failed to prepare forgotten favorites query: {}", e))?;
        let rows = stmt
            .query_map(params![cutoff, limit], |row| row.get::<_, String>(0))
            .map_err(|e| format!("Failed to query forgotten favorites: {}", e))?;
        let mut albums = Vec::new();
        for row in rows {
            albums
                .push(row.map_err(|e| format!("Failed to read forgotten favorite row: {}", e))?);
        }
        Ok(albums)
    }
}
