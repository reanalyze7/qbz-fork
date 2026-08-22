//! Per-track read operations on `cached_tracks`.

use rusqlite::params;

use super::row_to_cached_track_info;
use super::schema::OfflineCacheDb;
use crate::types::CachedTrackInfo;

impl OfflineCacheDb {
    /// Check if a track is cached and ready
    pub fn is_cached(&self, track_id: u64) -> Result<bool, String> {
        let count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM cached_tracks WHERE track_id = ?1 AND status = 'ready'",
                params![track_id as i64],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to check cache: {}", e))?;

        Ok(count > 0)
    }

    /// Get file path for a cached track
    pub fn get_file_path(&self, track_id: u64) -> Result<Option<String>, String> {
        let result = self.conn.query_row(
            "SELECT file_path FROM cached_tracks WHERE track_id = ?1 AND status = 'ready'",
            params![track_id as i64],
            |row| row.get(0),
        );

        match result {
            Ok(path) => Ok(Some(path)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(format!("Failed to get file path: {}", e)),
        }
    }

    /// Get track info
    pub fn get_track(&self, track_id: u64) -> Result<Option<CachedTrackInfo>, String> {
        let result = self.conn.query_row(
            "SELECT track_id, title, artist, album, album_id, duration_secs, file_size_bytes, quality, bit_depth, sample_rate, status, progress_percent, error_message, created_at, last_accessed_at, artwork_path, file_path
             FROM cached_tracks WHERE track_id = ?1",
            params![track_id as i64],
            row_to_cached_track_info,
        );

        match result {
            Ok(info) => Ok(Some(info)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(format!("Failed to get track: {}", e)),
        }
    }

    /// Get all cached tracks
    pub fn get_all_tracks(&self) -> Result<Vec<CachedTrackInfo>, String> {
        let mut stmt = self.conn.prepare(
            "SELECT track_id, title, artist, album, album_id, duration_secs, file_size_bytes, quality, bit_depth, sample_rate, status, progress_percent, error_message, created_at, last_accessed_at, artwork_path, file_path
             FROM cached_tracks ORDER BY last_accessed_at DESC"
        ).map_err(|e| format!("Failed to prepare query: {}", e))?;

        let tracks = stmt
            .query_map([], row_to_cached_track_info)
            .map_err(|e| format!("Failed to query tracks: {}", e))?;

        let mut result = Vec::new();
        for track in tracks {
            result.push(track.map_err(|e| format!("Failed to read track: {}", e))?);
        }

        Ok(result)
    }
}
