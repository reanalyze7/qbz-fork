use rusqlite::params;

use crate::{LibraryError, LocalTrack};

use super::super::LibraryDatabase;

impl LibraryDatabase {
    /// Get tracks for a metadata-grouped album. The `metadata_key`
    /// matches what [`Self::get_albums_metadata_grouped`] returns for
    /// the album's `id` field.
    pub fn get_album_tracks_metadata(
        &self,
        metadata_key: &str,
    ) -> Result<Vec<LocalTrack>, LibraryError> {
        let group_key_expr = crate::album_grouping::metadata_group_key_sql_expression();
        let sql = format!(
            "SELECT {cols} FROM local_tracks
             WHERE {group_key} = ?
             ORDER BY disc_number, track_number, title",
            cols = Self::TRACK_COLUMNS,
            group_key = group_key_expr,
        );

        let mut stmt = self
            .conn
            .prepare(&sql)
            .map_err(|e| LibraryError::Database(e.to_string()))?;

        let rows = stmt
            .query_map(params![metadata_key], |row| Self::row_to_track(row))
            .map_err(|e| LibraryError::Database(e.to_string()))?;

        let mut tracks = Vec::new();
        for track in rows {
            tracks.push(track.map_err(|e| LibraryError::Database(e.to_string()))?);
        }
        Ok(tracks)
    }
}
