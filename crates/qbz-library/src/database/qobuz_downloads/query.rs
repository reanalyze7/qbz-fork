//! Read-only lookups over Qobuz-downloaded tracks.

use rusqlite::params;

use crate::{LibraryError, LocalTrack};

use super::super::LibraryDatabase;

impl LibraryDatabase {
    /// All offline-copy rows: `source = 'qobuz_download'` with a real Qobuz
    /// id — the same set the Local Library "Offline" source filter shows.
    /// Read-only; used by the offline favorites rail (B9) to find favorite
    /// tracks that are playable without Qobuz.
    pub fn get_qobuz_download_tracks(&self) -> Result<Vec<LocalTrack>, LibraryError> {
        let sql = format!(
            "SELECT {} FROM local_tracks \
             WHERE source = 'qobuz_download' AND qobuz_track_id IS NOT NULL \
             ORDER BY artist COLLATE NOCASE, album COLLATE NOCASE, track_number",
            Self::TRACK_COLUMNS
        );
        let mut stmt = self
            .conn
            .prepare(&sql)
            .map_err(|e| LibraryError::Database(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| Self::row_to_track(row))
            .map_err(|e| LibraryError::Database(e.to_string()))?;

        let mut tracks = Vec::new();
        for track in rows {
            tracks.push(track.map_err(|e| LibraryError::Database(e.to_string()))?);
        }
        Ok(tracks)
    }

    /// Check if a track exists by Qobuz track ID
    pub fn track_exists_by_qobuz_id(&self, qobuz_track_id: u64) -> Result<bool, LibraryError> {
        let count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM local_tracks WHERE qobuz_track_id = ?1",
                params![qobuz_track_id as i64],
                |row| row.get(0),
            )
            .map_err(|e| LibraryError::Database(e.to_string()))?;
        Ok(count > 0)
    }

    /// Repair a track by file_path - restores both qobuz_track_id and source
    /// This handles tracks that were damaged by scanner's INSERT OR REPLACE
    /// Returns true if the track was found and updated
    /// Note: Database source field remains 'qobuz_download' for compatibility
    pub fn repair_qobuz_cached_track_by_path(
        &self,
        qobuz_track_id: u64,
        file_path: &str,
    ) -> Result<bool, LibraryError> {
        let updated = self
            .conn
            .execute(
                "UPDATE local_tracks
             SET source = 'qobuz_download', qobuz_track_id = ?1
             WHERE file_path = ?2 AND (source IS NULL OR source != 'qobuz_download')",
                params![qobuz_track_id as i64, file_path],
            )
            .map_err(|e| {
                LibraryError::Database(format!("Failed to repair cached track by path: {}", e))
            })?;
        Ok(updated > 0)
    }

    /// Check if a track exists by file path (for repair matching)
    pub fn track_exists_by_path(&self, file_path: &str) -> Result<bool, LibraryError> {
        let count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM local_tracks WHERE file_path = ?1",
                params![file_path],
                |row| row.get(0),
            )
            .map_err(|e| LibraryError::Database(e.to_string()))?;
        Ok(count > 0)
    }
}
