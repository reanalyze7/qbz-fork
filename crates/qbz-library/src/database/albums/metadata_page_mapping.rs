//! Row-mapping and ORDER BY helpers for [`super::metadata_page`] — split
//! out purely to keep `metadata_page.rs` under the 130-line file cap.

use crate::LocalAlbum;

use super::super::LibraryDatabase;

/// Row shape yielded by the paginated query: the album plus the
/// window-function `COUNT(*) OVER ()` total for the whole filtered set.
pub(super) fn row_to_album_with_total(row: &rusqlite::Row) -> rusqlite::Result<(LocalAlbum, u64)> {
    let group_key: String = row.get(0)?;
    let title: String = row.get(1)?;
    let artist: String = row.get(2)?;
    let all_artists: String = row.get::<_, Option<String>>(3)?.unwrap_or_default();
    let artwork_path: Option<String> = row.get(6)?;
    let source_folders: Option<String> = row.get(12)?;
    let total: u64 = row.get::<_, i64>(14)? as u64;

    Ok((
        LocalAlbum {
            id: group_key,
            title,
            artist,
            all_artists,
            year: row.get(4)?,
            catalog_number: row.get(5)?,
            artwork_path,
            track_count: row.get(7)?,
            total_duration_secs: row.get(8)?,
            format: LibraryDatabase::parse_format(
                &row.get::<_, Option<String>>(9)?.unwrap_or_default(),
            ),
            bit_depth: row.get(10)?,
            sample_rate: row.get::<_, Option<f64>>(11)?.unwrap_or(44100.0),
            directory_path: String::new(),
            source_folders,
            source: row
                .get::<_, Option<String>>(13)?
                .unwrap_or_else(|| "user".to_string()),
        },
        total,
    ))
}

/// Validated ORDER BY allowlist so user input never reaches the SQL
/// string directly. The unknown-album sentinel always sorts last
/// regardless of mode.
pub(super) fn order_clause_for(sort_by: &str, sort_dir: &str) -> &'static str {
    match (sort_by, sort_dir) {
        ("title", "asc") => "(group_key = '__unknown_album__'), title COLLATE NOCASE",
        ("title", "desc") => "(group_key = '__unknown_album__'), title COLLATE NOCASE DESC",
        ("year", "asc") => {
            "(group_key = '__unknown_album__'), year IS NULL, year ASC, title COLLATE NOCASE"
        }
        ("year", "desc") => {
            "(group_key = '__unknown_album__'), year IS NULL, year DESC, title COLLATE NOCASE"
        }
        ("artist", "desc") => {
            "(group_key = '__unknown_album__'), artist COLLATE NOCASE DESC, title COLLATE NOCASE"
        }
        // Default = artist asc
        _ => "(group_key = '__unknown_album__'), artist COLLATE NOCASE, title COLLATE NOCASE",
    }
}
