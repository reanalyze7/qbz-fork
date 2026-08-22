use crate::{LibraryError, LocalAlbum};

use super::super::LibraryDatabase;
use super::metadata_grouped_query::metadata_grouped_query;

impl LibraryDatabase {
    /// Get all albums grouped by metadata (album + album_artist OR
    /// artist), with fallback to folder grouping for tracks with no
    /// usable album tag, and a single 'Unknown Album' bucket for total
    /// orphans.
    ///
    /// Mirrors the shape of [`Self::get_albums_with_full_filter`] but
    /// uses the metadata group key from
    /// [`crate::album_grouping::metadata_group_key_sql_expression`].
    /// Rows have `directory_path = ""` and `source_folders` populated
    /// with the comma-separated list of contributing folder keys (so
    /// the UI can show a tooltip when N folders > 1).
    ///
    /// `include_hidden` is currently ignored: the `album_settings.hidden`
    /// flag targets the FOLDER key, which does not map cleanly onto
    /// metadata-grouped rows. Revisit if user feedback asks for it.
    pub fn get_albums_metadata_grouped(
        &self,
        include_hidden: bool,
        include_qobuz_downloads: bool,
        exclude_network_folders: bool,
        group_mode: crate::album_grouping::AlbumGroupMode,
    ) -> Result<Vec<LocalAlbum>, LibraryError> {
        let _ = include_hidden;

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

        let query = metadata_grouped_query(&group_key_expr, source_filter, network_filter);

        let mut stmt = self
            .conn
            .prepare(&query)
            .map_err(|e| LibraryError::Database(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                let group_key: String = row.get(0)?;
                let album: String = row.get(1)?;
                let artist: String = row.get(2)?;
                let all_artists: String =
                    row.get::<_, Option<String>>(3)?.unwrap_or_default();
                let artwork_path: Option<String> = row.get(6)?;
                let source_folders: Option<String> = row.get(12)?;

                Ok(LocalAlbum {
                    id: group_key.clone(),
                    title: album,
                    artist,
                    all_artists,
                    year: row.get(4)?,
                    catalog_number: row.get(5)?,
                    artwork_path,
                    track_count: row.get(7)?,
                    total_duration_secs: row.get(8)?,
                    format: Self::parse_format(
                        &row.get::<_, Option<String>>(9)?.unwrap_or_default(),
                    ),
                    bit_depth: row.get(10)?,
                    sample_rate: row.get::<_, Option<f64>>(11)?.unwrap_or(44100.0),
                    directory_path: String::new(),
                    source_folders,
                    source: row
                        .get::<_, Option<String>>(13)?
                        .unwrap_or_else(|| "user".to_string()),
                })
            })
            .map_err(|e| LibraryError::Database(e.to_string()))?;

        let mut albums = Vec::new();
        for album in rows {
            albums.push(album.map_err(|e| LibraryError::Database(e.to_string()))?);
        }
        Ok(albums)
    }
}
