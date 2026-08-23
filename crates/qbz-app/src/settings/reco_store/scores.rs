use rusqlite::params;

use super::schema::RecoStore;

/// A scored entry to write into `reco_scores` (mirrors `RecoScoreEntry`).
#[derive(Debug, Clone)]
pub(super) struct RecoScoreEntry {
    pub(super) track_id: Option<u64>,
    pub(super) album_id: Option<String>,
    pub(super) artist_id: Option<u64>,
    pub(super) score: f64,
}

impl RecoStore {
    // ---- Scores (companion table, written by train()) ----

    pub(super) fn has_scores(&self, score_type: &str) -> Result<bool, String> {
        let count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM reco_scores WHERE score_type = ?",
                params![score_type],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to query reco scores count: {}", e))?;
        Ok(count > 0)
    }

    pub(super) fn get_scored_album_ids(
        &self,
        score_type: &str,
        limit: u32,
    ) -> Result<Vec<String>, String> {
        let mut stmt = self
            .conn
            .prepare(
                r#"
                SELECT album_id FROM reco_scores
                WHERE score_type = ? AND item_type = 'album' AND album_id IS NOT NULL
                ORDER BY score DESC LIMIT ?
                "#,
            )
            .map_err(|e| format!("Failed to prepare scored albums query: {}", e))?;
        let rows = stmt
            .query_map(params![score_type, limit], |row| row.get::<_, String>(0))
            .map_err(|e| format!("Failed to query scored albums: {}", e))?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|e| format!("Failed to read scored album row: {}", e))?);
        }
        Ok(out)
    }

    pub(super) fn get_scored_track_ids(
        &self,
        score_type: &str,
        limit: u32,
    ) -> Result<Vec<u64>, String> {
        let mut stmt = self
            .conn
            .prepare(
                r#"
                SELECT track_id FROM reco_scores
                WHERE score_type = ? AND item_type = 'track' AND track_id IS NOT NULL
                ORDER BY score DESC LIMIT ?
                "#,
            )
            .map_err(|e| format!("Failed to prepare scored tracks query: {}", e))?;
        let rows = stmt
            .query_map(params![score_type, limit], |row| row.get::<_, u64>(0))
            .map_err(|e| format!("Failed to query scored tracks: {}", e))?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|e| format!("Failed to read scored track row: {}", e))?);
        }
        Ok(out)
    }

    pub(super) fn get_scored_artist_scores(
        &self,
        score_type: &str,
        limit: u32,
    ) -> Result<Vec<(u64, f64)>, String> {
        let mut stmt = self
            .conn
            .prepare(
                r#"
                SELECT artist_id, score FROM reco_scores
                WHERE score_type = ? AND item_type = 'artist' AND artist_id IS NOT NULL
                ORDER BY score DESC LIMIT ?
                "#,
            )
            .map_err(|e| format!("Failed to prepare scored artists query: {}", e))?;
        let rows = stmt
            .query_map(params![score_type, limit], |row| {
                Ok((row.get::<_, u64>(0)?, row.get::<_, f64>(1)?))
            })
            .map_err(|e| format!("Failed to query scored artists: {}", e))?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|e| format!("Failed to read scored artist row: {}", e))?);
        }
        Ok(out)
    }

}
