//! SQL builder for [`super::metadata_page`] — split out purely to keep
//! `metadata_page.rs` under the 130-line file cap.

/// Build the paginated, sort/filter-aware metadata-grouped albums query.
/// See `LibraryDatabase::get_albums_metadata_page` for the semantics of
/// `order_clause` (already-validated ORDER BY fragment) and the `?1..?4`
/// bind slots (`has_search`, `search_like`, `limit`, `offset`).
pub(super) fn metadata_page_query(
    group_key_expr: &str,
    source_filter: &str,
    network_filter: &str,
    order_clause: &str,
) -> String {
    format!(
        r#"
        WITH grouped AS (
            SELECT
                {group_key} AS group_key,
                -- Prefer `album` (metadata tag) over
                -- `album_group_title` (scan-time snapshot, which
                -- falls back to folder name if metadata is
                -- missing). Fixes #411 — when album metadata is
                -- valid, the folder name was winning because
                -- COALESCE returned `album_group_title` first.
                COALESCE(
                    NULLIF(NULLIF(TRIM(album), ''), 'Unknown Album'),
                    album_group_title,
                    'Unknown Album'
                ) AS title,
                COALESCE(album_artist, artist, 'Unknown Artist') AS artist,
                year,
                catalog_number,
                artwork_path,
                duration_secs,
                format,
                bit_depth,
                sample_rate,
                album_group_key AS source_folder,
                COALESCE(source, 'user') AS source,
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
                END AS artist,
                GROUP_CONCAT(DISTINCT track_artist) AS all_artists,
                MIN(year) AS year,
                MIN(catalog_number) AS catalog_number,
                MAX(CASE WHEN artwork_path IS NOT NULL THEN artwork_path END) AS artwork,
                COUNT(*) AS track_count,
                SUM(duration_secs) AS total_duration,
                MAX(format) AS format,
                MAX(bit_depth) AS bit_depth,
                MAX(sample_rate) AS sample_rate,
                GROUP_CONCAT(DISTINCT source_folder) AS source_folders,
                MAX(source) AS source
            FROM grouped
            GROUP BY group_key
        ),
        filtered AS (
            SELECT * FROM aggregated
            WHERE ?1 = 0 OR (title LIKE ?2 OR artist LIKE ?2)
        )
        SELECT
            group_key, title, artist, all_artists, year, catalog_number,
            artwork, track_count, total_duration, format, bit_depth,
            sample_rate, source_folders, source,
            COUNT(*) OVER () AS total
        FROM filtered
        ORDER BY {order_clause}
        LIMIT ?3 OFFSET ?4
        "#,
        group_key = group_key_expr,
        source_filter = source_filter,
        network_filter = network_filter,
        order_clause = order_clause,
    )
}
