use crate::LibraryError;

use super::super::LibraryDatabase;
use super::metadata_page_mapping::{order_clause_for, row_to_album_with_total};
use super::metadata_page_query::metadata_page_query;

impl LibraryDatabase {
    /// Paginated, sort/filter-aware slice of metadata-grouped local
    /// albums. Designed to back the chunked-store + recycling-grid pool
    /// on the frontend: caller asks for `[offset, offset+limit)` and
    /// receives those rows plus the total count of rows matching the
    /// same filter (via `COUNT(*) OVER ()`).
    ///
    /// Sort: one of `"artist"` (default), `"title"`, `"year"`, paired
    /// with direction `"asc"` (default) or `"desc"`. Unknown values
    /// fall back to artist-ascending. Albums with no `year` always sink
    /// to the bottom for the year sort.
    ///
    /// Search: a non-empty `search` becomes a `LIKE '%pattern%'` match
    /// applied after aggregation against the album's title or artist
    /// (mirrors the legacy in-memory `matchesAlbumSearchFast`).
    pub fn get_albums_metadata_page(
        &self,
        offset: u64,
        limit: u64,
        search: Option<&str>,
        sort_by: &str,
        sort_dir: &str,
        include_qobuz_downloads: bool,
        exclude_network_folders: bool,
        group_mode: crate::album_grouping::AlbumGroupMode,
    ) -> Result<crate::models::AlbumsMetadataPage, LibraryError> {
        self.get_albums_metadata_page_inner(
            offset,
            limit,
            search,
            sort_by,
            sort_dir,
            include_qobuz_downloads,
            exclude_network_folders,
            group_mode,
        )
    }

    fn get_albums_metadata_page_inner(
        &self,
        offset: u64,
        limit: u64,
        search: Option<&str>,
        sort_by: &str,
        sort_dir: &str,
        include_qobuz_downloads: bool,
        exclude_network_folders: bool,
        group_mode: crate::album_grouping::AlbumGroupMode,
    ) -> Result<crate::models::AlbumsMetadataPage, LibraryError> {
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

        let order_clause = order_clause_for(sort_by, sort_dir);
        let group_key_expr = crate::album_grouping::group_key_sql_expression(group_mode);

        let search_pattern = search.unwrap_or("").trim();
        let has_search: i64 = if search_pattern.is_empty() { 0 } else { 1 };
        let search_like = format!("%{}%", search_pattern);

        let query = metadata_page_query(&group_key_expr, source_filter, network_filter, order_clause);

        let mut stmt = self
            .conn
            .prepare(&query)
            .map_err(|e| LibraryError::Database(e.to_string()))?;

        let rows = stmt
            .query_map(
                rusqlite::params![has_search, search_like, limit as i64, offset as i64],
                row_to_album_with_total,
            )
            .map_err(|e| LibraryError::Database(e.to_string()))?;

        let mut albums = Vec::new();
        let mut total: u64 = 0;
        for row_result in rows {
            let (album, t) = row_result.map_err(|e| LibraryError::Database(e.to_string()))?;
            total = t;
            albums.push(album);
        }

        // Empty page (offset past the end or filter matches nothing).
        // The window-function trick gives us total only on returned
        // rows, so when there are none we have to ask separately.
        if albums.is_empty() {
            total = self.count_albums_metadata_for_page(
                search,
                include_qobuz_downloads,
                exclude_network_folders,
                group_mode,
            )?;
        }

        Ok(crate::models::AlbumsMetadataPage { albums, total })
    }
}
