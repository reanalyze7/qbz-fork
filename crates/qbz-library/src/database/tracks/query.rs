use rusqlite::{params, OptionalExtension};

use crate::{LibraryError, LocalTrack};

use super::super::LibraryDatabase;

impl LibraryDatabase {
    /// Get a track by ID
    pub fn get_track(&self, id: i64) -> Result<Option<LocalTrack>, LibraryError> {
        let mut stmt = self
            .conn
            .prepare(&format!(
                "SELECT {} FROM local_tracks WHERE id = ?",
                Self::TRACK_COLUMNS
            ))
            .map_err(|e| LibraryError::Database(e.to_string()))?;

        stmt.query_row(params![id], |row| Self::row_to_track(row))
            .optional()
            .map_err(|e| LibraryError::Database(e.to_string()))
    }

    /// Get a track by file path (for non-CUE tracks)
    pub fn get_track_by_path(&self, path: &str) -> Result<Option<LocalTrack>, LibraryError> {
        let mut stmt = self
            .conn
            .prepare(&format!(
                "SELECT {} FROM local_tracks WHERE file_path = ? AND cue_file_path IS NULL",
                Self::TRACK_COLUMNS
            ))
            .map_err(|e| LibraryError::Database(e.to_string()))?;

        stmt.query_row(params![path], |row| Self::row_to_track(row))
            .optional()
            .map_err(|e| LibraryError::Database(e.to_string()))
    }

    /// Get all file paths for local tracks (for cleanup check)
    pub fn get_all_track_paths(&self) -> Result<Vec<(i64, String)>, LibraryError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, file_path FROM local_tracks WHERE source IS NULL OR source = 'user'",
            )
            .map_err(|e| LibraryError::Database(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| LibraryError::Database(e.to_string()))?;

        let mut paths = Vec::new();
        for row in rows {
            paths.push(row.map_err(|e| LibraryError::Database(e.to_string()))?);
        }
        Ok(paths)
    }
}
