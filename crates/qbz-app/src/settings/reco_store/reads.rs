use rusqlite::params;

use super::now_ts;
use super::schema::RecoStore;

impl RecoStore {
    // ---- Read APIs: tracks ----

    /// Most-recently-played distinct track IDs (mirrors `get_recent_track_ids`).
    pub fn get_recent_track_ids(&self, limit: u32) -> Result<Vec<u64>, String> {
        let mut stmt = self
            .conn
            .prepare(
                r#"
                SELECT track_id, MAX(created_at) AS last_played
                FROM reco_events
                WHERE event_type = 'play' AND track_id IS NOT NULL
                GROUP BY track_id
                ORDER BY last_played DESC
                LIMIT ?
                "#,
            )
            .map_err(|e| format!("Failed to prepare recent tracks query: {}", e))?;

        let rows = stmt
            .query_map(params![limit], |row| row.get::<_, u64>(0))
            .map_err(|e| format!("Failed to query recent tracks: {}", e))?;

        let mut tracks = Vec::new();
        for row in rows {
            tracks.push(row.map_err(|e| format!("Failed to read recent track row: {}", e))?);
        }
        Ok(tracks)
    }

    /// NEW: time-windowed recent track IDs — distinct play tracks whose most
    /// recent play is within the last `window_secs` seconds, newest first,
    /// capped at `limit`. Backs WeeklyQ's 7-day window (window_secs = 7*86400).
    pub fn get_recent_track_ids_since(
        &self,
        window_secs: i64,
        limit: u32,
    ) -> Result<Vec<u64>, String> {
        let since_ts = now_ts().saturating_sub(window_secs.max(0));
        let mut stmt = self
            .conn
            .prepare(
                r#"
                SELECT track_id, MAX(created_at) AS last_played
                FROM reco_events
                WHERE event_type = 'play' AND track_id IS NOT NULL AND created_at >= ?
                GROUP BY track_id
                ORDER BY last_played DESC
                LIMIT ?
                "#,
            )
            .map_err(|e| format!("Failed to prepare windowed recent tracks query: {}", e))?;

        let rows = stmt
            .query_map(params![since_ts, limit], |row| row.get::<_, u64>(0))
            .map_err(|e| format!("Failed to query windowed recent tracks: {}", e))?;

        let mut tracks = Vec::new();
        for row in rows {
            tracks
                .push(row.map_err(|e| format!("Failed to read windowed recent track row: {}", e))?);
        }
        Ok(tracks)
    }

    /// Most-recently-favorited distinct track IDs (mirrors `get_favorite_track_ids`).
    pub fn get_favorite_track_ids(&self, limit: u32) -> Result<Vec<u64>, String> {
        let mut stmt = self
            .conn
            .prepare(
                r#"
                SELECT track_id, MAX(created_at) AS last_favorite
                FROM reco_events
                WHERE event_type = 'favorite' AND track_id IS NOT NULL
                GROUP BY track_id
                ORDER BY last_favorite DESC
                LIMIT ?
                "#,
            )
            .map_err(|e| format!("Failed to prepare favorite tracks query: {}", e))?;

        let rows = stmt
            .query_map(params![limit], |row| row.get::<_, u64>(0))
            .map_err(|e| format!("Failed to query favorite tracks: {}", e))?;

        let mut tracks = Vec::new();
        for row in rows {
            tracks.push(row.map_err(|e| format!("Failed to read favorite track row: {}", e))?);
        }
        Ok(tracks)
    }
}
