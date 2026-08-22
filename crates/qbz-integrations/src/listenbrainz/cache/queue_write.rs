//! Listen-queue writes: enqueue, mark sent, retry accounting, clear.

use super::ListenBrainzCache;

impl ListenBrainzCache {
    /// Queue a listen for later submission
    #[allow(clippy::too_many_arguments)]
    pub fn queue_listen(
        &self,
        listened_at: i64,
        artist: &str,
        track: &str,
        album: Option<&str>,
        recording_mbid: Option<&str>,
        release_mbid: Option<&str>,
        artist_mbids: Option<&[String]>,
        isrc: Option<&str>,
        duration_ms: Option<u64>,
    ) -> Result<i64, String> {
        let artist_mbids_json =
            artist_mbids.map(|ids| serde_json::to_string(ids).unwrap_or_default());

        self.conn
            .execute(
                "INSERT INTO listen_queue (listened_at, artist_name, track_name, release_name, recording_mbid, release_mbid, artist_mbids, isrc, duration_ms)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
                rusqlite::params![
                    listened_at,
                    artist,
                    track,
                    album,
                    recording_mbid,
                    release_mbid,
                    artist_mbids_json,
                    isrc,
                    duration_ms.map(|d| d as i64),
                ],
            )
            .map_err(|e| format!("Failed to queue listen: {}", e))?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Mark a listen as sent
    pub fn mark_sent(&self, id: i64) -> Result<(), String> {
        self.conn
            .execute("UPDATE listen_queue SET sent = 1 WHERE id = ?", [id])
            .map_err(|e| format!("Failed to mark sent: {}", e))?;
        Ok(())
    }

    /// Increment attempt count
    pub fn increment_attempts(&self, id: i64) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE listen_queue SET attempts = attempts + 1 WHERE id = ?",
                [id],
            )
            .map_err(|e| format!("Failed to increment attempts: {}", e))?;
        Ok(())
    }

    /// Batch mark multiple listens as sent
    pub fn mark_listens_sent(&self, ids: &[i64]) -> Result<(), String> {
        if ids.is_empty() {
            return Ok(());
        }
        let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
        let sql = format!(
            "UPDATE listen_queue SET sent = 1 WHERE id IN ({})",
            placeholders.join(", ")
        );
        let params: Vec<Box<dyn rusqlite::types::ToSql>> = ids
            .iter()
            .map(|id| Box::new(*id) as Box<dyn rusqlite::types::ToSql>)
            .collect();
        let params_refs: Vec<&dyn rusqlite::types::ToSql> =
            params.iter().map(|p| p.as_ref()).collect();
        self.conn
            .execute(&sql, params_refs.as_slice())
            .map_err(|e| format!("Failed to batch mark sent: {}", e))?;
        Ok(())
    }

    /// Clear all queued listens
    pub fn clear_queue(&self) -> Result<(), String> {
        self.conn
            .execute("DELETE FROM listen_queue", [])
            .map_err(|e| format!("Failed to clear queue: {}", e))?;
        Ok(())
    }
}
