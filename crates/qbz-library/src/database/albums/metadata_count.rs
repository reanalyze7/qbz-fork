use super::super::LibraryDatabase;
use crate::LibraryError;

impl LibraryDatabase {
    /// Companion to `get_albums_metadata_page` — total count of albums
    /// matching the same filter. Used when the page is empty (so the
    /// window-function-derived total isn't available) or when the
    /// frontend wants to know the count before requesting any page.
    pub(super) fn count_albums_metadata_for_page(
        &self,
        search: Option<&str>,
        include_qobuz_downloads: bool,
        exclude_network_folders: bool,
        group_mode: crate::album_grouping::AlbumGroupMode,
    ) -> Result<u64, LibraryError> {
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
        let group_key_expr = crate::album_grouping::group_key_sql_expression(group_mode);
        let search_pattern = search.unwrap_or("").trim();
        let has_search: i64 = if search_pattern.is_empty() { 0 } else { 1 };
        let search_like = format!("%{}%", search_pattern);

        let query = format!(
            r#"
            WITH grouped AS (
                SELECT
                    {group_key} AS group_key,
                    COALESCE(
                        NULLIF(NULLIF(TRIM(album), ''), 'Unknown Album'),
                        album_group_title,
                        'Unknown Album'
                    ) AS title,
                    COALESCE(album_artist, artist, 'Unknown Artist') AS artist,
                    artist AS track_artist
                FROM local_tracks
                WHERE 1=1 {source_filter} {network_filter}
            ),
            aggregated AS (
                SELECT
                    group_key,
                    CASE WHEN group_key = '__unknown_album__'
                         THEN 'Unknown Album'
                         ELSE MIN(title)
                    END AS title,
                    CASE WHEN COUNT(DISTINCT track_artist) > 1
                         THEN 'Various Artists'
                         ELSE MIN(artist)
                    END AS artist
                FROM grouped
                GROUP BY group_key
            )
            SELECT COUNT(*)
            FROM aggregated
            WHERE ?1 = 0 OR (title LIKE ?2 OR artist LIKE ?2)
            "#,
            group_key = group_key_expr,
            source_filter = source_filter,
            network_filter = network_filter,
        );

        let total: i64 = self
            .conn
            .query_row(
                &query,
                rusqlite::params![has_search, search_like],
                |row| row.get(0),
            )
            .map_err(|e| LibraryError::Database(e.to_string()))?;
        Ok(total as u64)
    }
}
