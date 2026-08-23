use super::types::QueuedScrobble;
use super::OfflineModeStore;
use rusqlite::params;

impl OfflineModeStore {
    // === Last.fm scrobble queue ===
    //
    // Minimal API over the `scrobble_queue` table the base schema already
    // creates. SQL kept IDENTICAL to Tauri's `src-tauri/src/offline/mod.rs`
    // scrobble-queue methods so both frontends interoperate on the same rows.

    /// Queue a scrobble for later submission to Last.fm.
    pub fn queue_scrobble(
        &self,
        artist: &str,
        track: &str,
        album: Option<&str>,
        timestamp: i64,
    ) -> Result<i64, String> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        self.conn
            .execute(
                "INSERT INTO scrobble_queue (artist, track, album, timestamp, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![artist, track, album, timestamp, now],
            )
            .map_err(|e| format!("Failed to queue scrobble: {}", e))?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Unsent scrobbles, oldest first (cap at 50 — the Last.fm batch limit).
    pub fn get_queued_scrobbles(&self, limit: u32) -> Result<Vec<QueuedScrobble>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, artist, track, album, timestamp, created_at, sent
                 FROM scrobble_queue WHERE sent = 0 ORDER BY timestamp ASC LIMIT ?1",
            )
            .map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let scrobbles = stmt
            .query_map(params![limit], |row| {
                Ok(QueuedScrobble {
                    id: row.get(0)?,
                    artist: row.get(1)?,
                    track: row.get(2)?,
                    album: row.get(3)?,
                    timestamp: row.get(4)?,
                    created_at: row.get(5)?,
                    sent: row.get::<_, i64>(6)? != 0,
                })
            })
            .map_err(|e| format!("Failed to query queued scrobbles: {}", e))?;

        scrobbles
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect queued scrobbles: {}", e))
    }

    /// Batch mark scrobbles as sent.
    pub fn mark_scrobbles_sent(&self, ids: &[i64]) -> Result<(), String> {
        if ids.is_empty() {
            return Ok(());
        }

        let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
        let sql = format!(
            "UPDATE scrobble_queue SET sent = 1 WHERE id IN ({})",
            placeholders.join(",")
        );

        let mut stmt = self
            .conn
            .prepare(&sql)
            .map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let sql_params: Vec<&dyn rusqlite::ToSql> =
            ids.iter().map(|id| id as &dyn rusqlite::ToSql).collect();
        stmt.execute(sql_params.as_slice())
            .map_err(|e| format!("Failed to mark scrobbles as sent: {}", e))?;

        Ok(())
    }

    /// Delete sent scrobbles older than `older_than_days` (post-flush cleanup).
    pub fn cleanup_sent_scrobbles(&self, older_than_days: u32) -> Result<u32, String> {
        let cutoff = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0)
            - (older_than_days as i64 * 24 * 60 * 60);

        let deleted = self
            .conn
            .execute(
                "DELETE FROM scrobble_queue WHERE sent = 1 AND created_at < ?1",
                params![cutoff],
            )
            .map_err(|e| format!("Failed to cleanup sent scrobbles: {}", e))?;

        Ok(deleted as u32)
    }

    /// Count of queued (unsent) scrobbles.
    pub fn queued_scrobble_count(&self) -> Result<u32, String> {
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM scrobble_queue WHERE sent = 0",
                [],
                |row| row.get::<_, i64>(0),
            )
            .map(|count| count as u32)
            .map_err(|e| format!("Failed to count queued scrobbles: {}", e))
    }
}
