//! Inserting new `cached_tracks` rows (single and batch).

use rusqlite::params;

use super::schema::OfflineCacheDb;
use crate::types::TrackCacheInfo;

impl OfflineCacheDb {
    /// Insert a new track to cache for offline
    pub fn insert_track(&self, info: &TrackCacheInfo, file_path: &str) -> Result<(), String> {
        self.conn.execute(
            "INSERT OR REPLACE INTO cached_tracks
             (track_id, title, artist, album, album_id, duration_secs, file_path, quality, bit_depth, sample_rate, status, progress_percent, created_at, last_accessed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 'queued', 0, datetime('now'), datetime('now'))",
            params![
                info.track_id as i64,
                info.title,
                info.artist,
                info.album,
                info.album_id,
                info.duration_secs as i64,
                file_path,
                info.quality,
                info.bit_depth.map(|v| v as i64),
                info.sample_rate,
            ],
        ).map_err(|e| format!("Failed to insert track: {}", e))?;

        Ok(())
    }

    /// Insert multiple tracks in a single transaction (batch queuing)
    pub fn insert_tracks_batch(&self, tracks: &[(&TrackCacheInfo, String)]) -> Result<(), String> {
        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(|e| format!("Failed to begin transaction: {}", e))?;

        {
            let mut stmt = tx.prepare(
                "INSERT OR REPLACE INTO cached_tracks
                 (track_id, title, artist, album, album_id, duration_secs, file_path, quality, bit_depth, sample_rate, status, progress_percent, created_at, last_accessed_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 'queued', 0, datetime('now'), datetime('now'))"
            ).map_err(|e| format!("Failed to prepare batch insert: {}", e))?;

            for (info, file_path) in tracks {
                stmt.execute(params![
                    info.track_id as i64,
                    info.title,
                    info.artist,
                    info.album,
                    info.album_id,
                    info.duration_secs as i64,
                    file_path,
                    info.quality,
                    info.bit_depth.map(|v| v as i64),
                    info.sample_rate,
                ])
                .map_err(|e| format!("Failed to insert track {}: {}", info.track_id, e))?;
            }
        }

        tx.commit()
            .map_err(|e| format!("Failed to commit batch insert: {}", e))?;

        Ok(())
    }
}
