//! SQL builder for [`super::filter_sql`] — split out purely to keep
//! `filter_sql.rs` under the 130-line file cap.

/// Build the folder-grouped albums query. Shared by both the
/// `include_hidden = true` and `include_hidden = false` branches of
/// `LibraryDatabase::get_albums_with_full_filter` — the only
/// difference is the extra `WHERE group_key NOT IN (...)` clause that
/// excludes albums hidden via `album_settings`.
pub(super) fn albums_query(
    include_hidden: bool,
    source_filter: &str,
    network_filter: &str,
) -> String {
    let hidden_filter = if include_hidden {
        ""
    } else {
        "WHERE group_key NOT IN (
            SELECT album_group_key FROM album_settings WHERE hidden = 1
        )"
    };
    format!(
        r#"
        SELECT
            group_key,
            MIN(title) as title,
            CASE
                WHEN COUNT(DISTINCT artist) > 1 THEN 'Various Artists'
                ELSE MIN(artist)
            END as artist,
            GROUP_CONCAT(DISTINCT artist) as all_artists,
            MIN(year) as year,
            MIN(catalog_number) as catalog_number,
            MAX(CASE WHEN artwork_path IS NOT NULL THEN artwork_path END) as artwork,
            COUNT(*) as track_count,
            SUM(duration_secs) as total_duration,
            MAX(format) as format,
            MAX(bit_depth) as bit_depth,
            MAX(sample_rate) as sample_rate,
            MAX(group_key) as directory_path,
            MAX(source) as source
        FROM (
            SELECT
                COALESCE(album_group_key, album || '|' || COALESCE(album_artist, artist)) as group_key,
                COALESCE(album_group_title, album) as title,
                COALESCE(album_artist, artist) as artist,
                year,
                catalog_number,
                artwork_path,
                duration_secs,
                format,
                bit_depth,
                sample_rate,
                COALESCE(source, 'user') as source
            FROM local_tracks
            WHERE 1=1 {source_filter} {network_filter}
        )
        {hidden_filter}
        GROUP BY group_key
        ORDER BY artist, title
        "#,
        source_filter = source_filter,
        network_filter = network_filter,
        hidden_filter = hidden_filter,
    )
}
