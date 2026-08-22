use rusqlite::params;

use crate::LibraryError;

use super::super::LibraryDatabase;

impl LibraryDatabase {
    /// Get count of local tracks in a playlist
    pub fn get_playlist_local_track_count(
        &self,
        qobuz_playlist_id: u64,
    ) -> Result<u32, LibraryError> {
        let count: u32 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM playlist_local_tracks WHERE qobuz_playlist_id = ?1",
                params![qobuz_playlist_id as i64],
                |row| row.get(0),
            )
            .map_err(|e| {
                LibraryError::Database(format!("Failed to count playlist local tracks: {}", e))
            })?;

        Ok(count)
    }

    /// Get local track counts for all playlists.
    ///
    /// "Local" here is the user-facing sense — anything that isn't a Qobuz
    /// server track: file-system local tracks (user / qobuz purchases /
    /// offline-cached downloads, all in local_tracks).
    pub fn get_all_playlist_local_track_counts(
        &self,
    ) -> Result<std::collections::HashMap<u64, u32>, LibraryError> {
        let mut result: std::collections::HashMap<u64, u32> = std::collections::HashMap::new();

        let mut stmt = self
            .conn
            .prepare(
                "SELECT qobuz_playlist_id, COUNT(*) as count
             FROM playlist_local_tracks
             GROUP BY qobuz_playlist_id",
            )
            .map_err(|e| LibraryError::Database(format!("Failed to prepare query: {}", e)))?;

        let rows = stmt
            .query_map([], |row| {
                let playlist_id: i64 = row.get(0)?;
                let count: u32 = row.get(1)?;
                Ok((playlist_id as u64, count))
            })
            .map_err(|e| LibraryError::Database(format!("Failed to query: {}", e)))?;

        for row in rows {
            let (playlist_id, count) =
                row.map_err(|e| LibraryError::Database(format!("Failed to read row: {}", e)))?;
            result.insert(playlist_id, count);
        }

        Ok(result)
    }
}
