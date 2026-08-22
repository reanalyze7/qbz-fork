//! Whole-result-set track search (`search` / `search_with_filter`).
//! See [`super::paged`] for the paginated variant used by the Tracks tab.

use rusqlite::params;

use crate::LibraryError;

use super::super::LibraryDatabase;

impl LibraryDatabase {
    pub fn search(&self, query: &str, limit: u32) -> Result<Vec<crate::LocalTrack>, LibraryError> {
        self.search_with_filter(query, limit, true, false)
    }

    /// Search tracks with filter options
    /// This filters directly in SQL to avoid post-query filtering overhead
    pub fn search_with_filter(
        &self,
        query: &str,
        limit: u32,
        include_qobuz_downloads: bool,
        exclude_network_folders: bool,
    ) -> Result<Vec<crate::LocalTrack>, LibraryError> {
        let pattern = format!("%{}%", query);

        let source_filter = if include_qobuz_downloads {
            ""
        } else {
            "AND (source IS NULL OR source != 'qobuz_download')"
        };

        let network_filter = if exclude_network_folders {
            "AND NOT EXISTS (
                SELECT 1 FROM library_folders nf
                WHERE nf.is_network = 1
                AND local_tracks.file_path LIKE nf.path || '%'
            )"
        } else {
            ""
        };

        // limit = 0 means no limit (fetch all)
        let limit_clause = if limit == 0 {
            String::new()
        } else {
            format!("LIMIT {}", limit)
        };

        // ORDER BY matches the album-grouped browsing the Tracks tab uses
        // by default. Sorting in SQLite is sub-100ms for 100K rows; doing it
        // in JS with localeCompare on the same volume blocks the main thread
        // for several seconds per pass.
        let sql = format!(
            "SELECT {} FROM local_tracks \
             WHERE (title LIKE ?1 OR artist LIKE ?1 OR album LIKE ?1) \
             {} {} \
             ORDER BY album COLLATE NOCASE, \
                      COALESCE(album_artist, artist) COLLATE NOCASE, \
                      disc_number, \
                      track_number, \
                      title COLLATE NOCASE \
             {}",
            Self::TRACK_COLUMNS,
            source_filter,
            network_filter,
            limit_clause
        );

        let mut stmt = self
            .conn
            .prepare(&sql)
            .map_err(|e| LibraryError::Database(e.to_string()))?;

        let rows = stmt
            .query_map(params![&pattern], |row| Self::row_to_track(row))
            .map_err(|e| LibraryError::Database(e.to_string()))?;

        let mut tracks = Vec::new();
        for track in rows {
            tracks.push(track.map_err(|e| LibraryError::Database(e.to_string()))?);
        }
        Ok(tracks)
    }
}
