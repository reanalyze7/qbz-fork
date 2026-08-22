//! SQL builder for [`super::children`] — split out purely to keep
//! `children.rs` under the 130-line file cap.

/// Build the "direct children of a folder" query.
///
/// SQL strategy (CTE form for readability; SQLite uses
/// idx_tracks_file_path on the LIKE prefix in the candidates step):
///   suffix        = file_path with the parent prefix + '/' stripped
///   child_segment = leading path component of suffix
///   kind          = 'folder' if suffix contains a '/', else 'track'
/// Group by (child_segment, kind) so folders aggregate over all
/// descendant tracks; track rows are 1:1 with their file. Include
/// MIN(file_path) so we can recover the absolute path for tracks
/// (folders ignore it and reconstruct path from parent + segment).
pub(super) fn children_query(network_filter: &str) -> String {
    format!(
        "WITH candidates AS ( \
            SELECT \
                substr(file_path, length(?1) + 2) AS suffix, \
                file_path, \
                artwork_path \
            FROM local_tracks \
            WHERE file_path LIKE ?2 || '/%' ESCAPE '\\' \
              AND COALESCE(source, 'user') = 'user' \
              {network_filter} \
         ), \
         classified AS ( \
            SELECT \
                CASE WHEN instr(suffix, '/') > 0 \
                     THEN substr(suffix, 1, instr(suffix, '/') - 1) \
                     ELSE suffix \
                END AS child_segment, \
                CASE WHEN instr(suffix, '/') > 0 \
                     THEN 'folder' ELSE 'track' \
                END AS kind, \
                file_path, \
                artwork_path \
            FROM candidates \
         ) \
         SELECT \
            child_segment, \
            kind, \
            COUNT(*) AS track_count_under, \
            MAX(artwork_path) AS artwork, \
            MIN(file_path) AS one_file_path \
         FROM classified \
         GROUP BY child_segment, kind",
        network_filter = network_filter,
    )
}
