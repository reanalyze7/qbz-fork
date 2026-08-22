//! Match Qobuz tracks to locally-cached copies, by Qobuz id or by fuzzy
//! (lowercased title/artist/album) metadata match.

use rusqlite::{params, OptionalExtension};

use crate::LibraryError;

use super::super::LibraryDatabase;

impl LibraryDatabase {
    /// Check if a track exists locally by Qobuz track ID
    pub fn has_local_track_by_qobuz_id(&self, qobuz_track_id: u64) -> Result<bool, LibraryError> {
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

    /// Check if a track exists locally by title, artist, and album (fuzzy match)
    pub fn has_local_track_by_metadata(
        &self,
        title: &str,
        artist: &str,
        album: &str,
    ) -> Result<bool, LibraryError> {
        // Normalize strings for comparison
        let title_lower = title.to_lowercase();
        let artist_lower = artist.to_lowercase();
        let album_lower = album.to_lowercase();

        let count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM local_tracks
                 WHERE LOWER(title) = ?1 AND LOWER(artist) = ?2 AND LOWER(album) = ?3",
                params![title_lower, artist_lower, album_lower],
                |row| row.get(0),
            )
            .map_err(|e| LibraryError::Database(e.to_string()))?;
        Ok(count > 0)
    }

    /// Get local track ID by Qobuz track ID (for downloaded tracks)
    pub fn get_local_track_id_by_qobuz_id(
        &self,
        qobuz_track_id: u64,
    ) -> Result<Option<i64>, LibraryError> {
        self.conn
            .query_row(
                "SELECT id FROM local_tracks WHERE qobuz_track_id = ?1",
                params![qobuz_track_id as i64],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| LibraryError::Database(e.to_string()))
    }

    /// Get local track ID by metadata (title, artist, album)
    pub fn get_local_track_id_by_metadata(
        &self,
        title: &str,
        artist: &str,
        album: &str,
    ) -> Result<Option<i64>, LibraryError> {
        let title_lower = title.to_lowercase();
        let artist_lower = artist.to_lowercase();
        let album_lower = album.to_lowercase();

        self.conn
            .query_row(
                "SELECT id FROM local_tracks
                 WHERE LOWER(title) = ?1 AND LOWER(artist) = ?2 AND LOWER(album) = ?3
                 LIMIT 1",
                params![title_lower, artist_lower, album_lower],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| LibraryError::Database(e.to_string()))
    }

}
