//! Paginated track search — the performant path for the Tracks tab.
//! See [`super::query`] for the whole-result-set variant.

use rusqlite::params;

use crate::LibraryError;

use super::super::LibraryDatabase;

impl LibraryDatabase {
    /// Paged variant of `search_with_filter` — the performant path for the
    /// Tracks tab. Returns one page (`LIMIT`/`OFFSET`) in the `sort` order,
    /// so the frontend never materializes the whole table (the documented
    /// ~16K-row freeze). An empty `query` matches everything.
    pub fn search_with_filter_page(
        &self,
        query: &str,
        offset: u64,
        limit: u64,
        include_qobuz_downloads: bool,
        exclude_network_folders: bool,
        sort: &str,
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
        // ORDER BY clause is built from a validated allowlist so user
        // input never reaches the SQL string directly. NULL years always
        // sort last regardless of direction; the fallback ("default" or
        // any unknown key) is the historical album-grouped order. Every
        // explicit key ends in `id` so LIMIT/OFFSET pagination is
        // deterministic across ties (mass ties are real: a batch scan
        // shares one indexed_at).
        let order_clause = match sort {
            "title-asc" => "title COLLATE NOCASE, artist COLLATE NOCASE, id",
            "title-desc" => "title COLLATE NOCASE DESC, artist COLLATE NOCASE, id",
            "artist-asc" => "COALESCE(album_artist, artist) COLLATE NOCASE, album COLLATE NOCASE, disc_number, track_number, id",
            "artist-desc" => "COALESCE(album_artist, artist) COLLATE NOCASE DESC, album COLLATE NOCASE, disc_number, track_number, id",
            "year-desc" => "year IS NULL, year DESC, album COLLATE NOCASE, disc_number, track_number, id",
            "year-asc" => "year IS NULL, year ASC, album COLLATE NOCASE, disc_number, track_number, id",
            "added-desc" => "indexed_at DESC, album COLLATE NOCASE, disc_number, track_number, id",
            // Default = the pre-sort hardcoded order (album-grouped).
            _ => "album COLLATE NOCASE, \
                  COALESCE(album_artist, artist) COLLATE NOCASE, \
                  disc_number, \
                  track_number, \
                  title COLLATE NOCASE",
        };
        let sql = format!(
            "SELECT {} FROM local_tracks \
             WHERE (title LIKE ?1 OR artist LIKE ?1 OR album LIKE ?1) \
             {} {} \
             ORDER BY {} \
             LIMIT ?2 OFFSET ?3",
            Self::TRACK_COLUMNS,
            source_filter,
            network_filter,
            order_clause,
        );
        let mut stmt = self
            .conn
            .prepare(&sql)
            .map_err(|e| LibraryError::Database(e.to_string()))?;
        let rows = stmt
            .query_map(params![&pattern, limit as i64, offset as i64], |row| {
                Self::row_to_track(row)
            })
            .map_err(|e| LibraryError::Database(e.to_string()))?;
        let mut tracks = Vec::new();
        for track in rows {
            tracks.push(track.map_err(|e| LibraryError::Database(e.to_string()))?);
        }
        Ok(tracks)
    }
}
