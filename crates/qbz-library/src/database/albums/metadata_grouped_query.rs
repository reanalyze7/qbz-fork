//! SQL builder for [`super::metadata_grouped`] — split out purely to keep
//! `metadata_grouped.rs` under the 130-line file cap.

/// Build the metadata-grouped albums query (album + album_artist OR
/// artist grouping, with folder/orphan fallback). See
/// `LibraryDatabase::get_albums_metadata_grouped` for the semantics.
pub(super) fn metadata_grouped_query(
    group_key_expr: &str,
    source_filter: &str,
    network_filter: &str,
) -> String {
    format!(
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
                year,
                catalog_number,
                artwork_path,
                duration_secs,
                format,
                bit_depth,
                sample_rate,
                album_group_key AS source_folder,
                COALESCE(source, 'user') AS source
            FROM local_tracks
            WHERE 1=1 {source_filter} {network_filter}
        )
        SELECT
            group_key,
            CASE WHEN group_key = '__unknown_album__'
                 THEN 'Unknown Album'
                 ELSE MIN(title)
            END AS title,
            CASE WHEN COUNT(DISTINCT artist) > 1
                 THEN 'Various Artists'
                 ELSE MIN(artist)
            END AS artist,
            GROUP_CONCAT(DISTINCT artist) AS all_artists,
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
        ORDER BY (group_key = '__unknown_album__'), artist, title
        "#,
        group_key = group_key_expr,
        source_filter = source_filter,
        network_filter = network_filter,
    )
}
