//! Query for syncing ready-cached tracks into the local library.

use super::schema::OfflineCacheDb;
use crate::types::ReadyTrackForSync;

impl OfflineCacheDb {
    /// Get all ready (cached) tracks with their file paths for syncing to library
    pub fn get_ready_tracks_for_sync(&self) -> Result<Vec<ReadyTrackForSync>, String> {
        let mut stmt = self.conn.prepare(
            "SELECT track_id, title, artist, album, duration_secs, file_path, bit_depth, sample_rate
             FROM cached_tracks WHERE status = 'ready'"
        ).map_err(|e| format!("Failed to prepare query: {}", e))?;

        let tracks = stmt
            .query_map([], |row| {
                Ok(ReadyTrackForSync {
                    track_id: row.get::<_, i64>(0)? as u64,
                    title: row.get(1)?,
                    artist: row.get(2)?,
                    album: row.get(3)?,
                    duration_secs: row.get::<_, i64>(4)? as u64,
                    file_path: row.get(5)?,
                    bit_depth: row.get::<_, Option<i64>>(6)?.map(|v| v as u32),
                    sample_rate: row.get(7)?,
                })
            })
            .map_err(|e| format!("Failed to query tracks: {}", e))?;

        let mut result = Vec::new();
        for track in tracks {
            result.push(track.map_err(|e| format!("Failed to read track: {}", e))?);
        }

        Ok(result)
    }
}
