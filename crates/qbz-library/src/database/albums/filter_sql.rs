use crate::{LibraryError, LocalAlbum};

use super::super::LibraryDatabase;
use super::filter_sql_query::albums_query;

impl LibraryDatabase {
    /// Get all albums with full filter options including network folder exclusion
    /// This method filters network folders directly in SQL to avoid N+1 query patterns
    pub fn get_albums_with_full_filter(
        &self,
        include_hidden: bool,
        include_qobuz_downloads: bool,
        exclude_network_folders: bool,
    ) -> Result<Vec<LocalAlbum>, LibraryError> {
        let source_filter = if include_qobuz_downloads {
            ""
        } else {
            "AND (source IS NULL OR source != 'qobuz_download')"
        };

        // Network folder filter: exclude tracks whose file_path starts with any network folder path
        let network_filter = if exclude_network_folders {
            "AND NOT EXISTS (
                SELECT 1 FROM library_folders nf
                WHERE nf.is_network = 1
                AND local_tracks.file_path LIKE nf.path || '%'
            )"
        } else {
            ""
        };

        let query = albums_query(include_hidden, source_filter, network_filter);

        let mut stmt = self
            .conn
            .prepare(&query)
            .map_err(|e| LibraryError::Database(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                let group_key: String = row.get(0)?;
                let album: String = row.get(1)?;
                let artist: String = row.get(2)?;
                let all_artists: String = row.get::<_, Option<String>>(3)?.unwrap_or_default();
                let artwork_path: Option<String> = row.get(6)?;

                log::debug!(
                    "Album {} by {}: artwork_path = {:?}",
                    album,
                    artist,
                    artwork_path
                );

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
                    directory_path: row
                        .get::<_, Option<String>>(12)?
                        .unwrap_or_else(|| group_key.clone()),
                    source_folders: None,
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

