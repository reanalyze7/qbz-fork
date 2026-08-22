//! Listen-queue reads: pending listens, counts, cleanup of old sent rows.

use rusqlite::Result as SqlResult;

use super::ListenBrainzCache;
use crate::listenbrainz::models::QueuedListen;

impl ListenBrainzCache {
    /// Get pending listens (not yet sent)
    pub fn get_pending_listens(&self, limit: u32) -> Result<Vec<QueuedListen>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, listened_at, artist_name, track_name, release_name,
                        recording_mbid, release_mbid, artist_mbids, isrc, duration_ms,
                        created_at, attempts, sent
                 FROM listen_queue
                 WHERE sent = 0
                 ORDER BY listened_at ASC
                 LIMIT ?",
            )
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let listens = stmt
            .query_map([limit], |row| {
                let artist_mbids_json: Option<String> = row.get(7)?;
                let artist_mbids =
                    artist_mbids_json.and_then(|json| serde_json::from_str(&json).ok());

                Ok(QueuedListen {
                    id: row.get(0)?,
                    listened_at: row.get(1)?,
                    artist_name: row.get(2)?,
                    track_name: row.get(3)?,
                    release_name: row.get(4)?,
                    recording_mbid: row.get(5)?,
                    release_mbid: row.get(6)?,
                    artist_mbids,
                    isrc: row.get(8)?,
                    duration_ms: row.get::<_, Option<i64>>(9)?.map(|d| d as u64),
                    created_at: row.get(10)?,
                    attempts: row.get(11)?,
                    sent: row.get::<_, i32>(12)? != 0,
                })
            })
            .map_err(|e| format!("Failed to query listens: {}", e))?
            .collect::<SqlResult<Vec<_>>>()
            .map_err(|e| format!("Failed to collect listens: {}", e))?;

        Ok(listens)
    }

    /// Get count of unsent listens in queue
    pub fn get_queue_count(&self) -> Result<u32, String> {
        let count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM listen_queue WHERE sent = 0",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        Ok(count as u32)
    }

    /// Delete old sent listens
    pub fn cleanup_sent(&self, older_than_days: u32) -> Result<u64, String> {
        let cutoff = chrono::Utc::now().timestamp() - (older_than_days as i64 * 86400);
        let deleted = self
            .conn
            .execute(
                "DELETE FROM listen_queue WHERE sent = 1 AND created_at < ?",
                [cutoff],
            )
            .map_err(|e| format!("Failed to cleanup: {}", e))?;
        Ok(deleted as u64)
    }
}
