use rusqlite::params;

use super::now_ts;
use super::schema::RecoStore;
use super::scores::RecoScoreEntry;

impl RecoStore {
    pub(super) fn replace_scores(
        &mut self,
        score_type: &str,
        item_type: &str,
        entries: &[RecoScoreEntry],
    ) -> Result<(), String> {
        let updated_at = now_ts();
        let tx = self
            .conn
            .transaction()
            .map_err(|e| format!("Failed to start reco scores transaction: {}", e))?;

        tx.execute(
            "DELETE FROM reco_scores WHERE score_type = ? AND item_type = ?",
            params![score_type, item_type],
        )
        .map_err(|e| format!("Failed to clear reco scores: {}", e))?;

        if !entries.is_empty() {
            let mut stmt = tx
                .prepare(
                    r#"
                    INSERT INTO reco_scores
                        (score_type, item_type, track_id, album_id, artist_id, score, updated_at)
                    VALUES (?, ?, ?, ?, ?, ?, ?)
                    "#,
                )
                .map_err(|e| format!("Failed to prepare reco scores insert: {}", e))?;
            for entry in entries {
                stmt.execute(params![
                    score_type,
                    item_type,
                    entry.track_id,
                    entry.album_id.as_deref(),
                    entry.artist_id,
                    entry.score,
                    updated_at,
                ])
                .map_err(|e| format!("Failed to insert reco score: {}", e))?;
            }
        }

        tx.commit()
            .map_err(|e| format!("Failed to commit reco scores: {}", e))?;
        Ok(())
    }
}
