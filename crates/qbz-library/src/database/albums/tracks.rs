use rusqlite::params;

use crate::{LibraryError, LocalTrack};

use super::super::LibraryDatabase;

impl LibraryDatabase {
    /// Get tracks for an album group
    pub fn get_album_tracks(&self, group_key: &str) -> Result<Vec<LocalTrack>, LibraryError> {
        let sql = format!(
            "SELECT {} FROM local_tracks \
             WHERE COALESCE(album_group_key, album || '|' || COALESCE(album_artist, artist)) = ? \
             ORDER BY disc_number, track_number, title",
            Self::TRACK_COLUMNS
        );
        let mut stmt = self
            .conn
            .prepare(&sql)
            .map_err(|e| LibraryError::Database(e.to_string()))?;

        let rows = stmt
            .query_map(params![group_key], |row| Self::row_to_track(row))
            .map_err(|e| LibraryError::Database(e.to_string()))?;

        let mut tracks = Vec::new();
        for track in rows {
            tracks.push(track.map_err(|e| LibraryError::Database(e.to_string()))?);
        }
        Ok(tracks)
    }
}
