//! Artist listing queries — filters directly in SQL (rather than
//! post-query grouping) to avoid N+1 query patterns when the library is
//! large.

use crate::{LibraryError, LocalArtist};

use super::LibraryDatabase;

impl LibraryDatabase {
    /// Get all artists
    pub fn get_artists(&self) -> Result<Vec<LocalArtist>, LibraryError> {
        self.get_artists_with_filter(true, false)
    }

    /// Get all artists with filter options
    /// This filters directly in SQL to avoid N+1 query patterns
    pub fn get_artists_with_filter(
        &self,
        include_qobuz_downloads: bool,
        exclude_network_folders: bool,
    ) -> Result<Vec<LocalArtist>, LibraryError> {
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

        let query = format!(
            r#"
            SELECT
                COALESCE(album_artist, artist) as name,
                COUNT(DISTINCT COALESCE(album_group_key, album || '|' || COALESCE(album_artist, artist))) as album_count,
                COUNT(*) as track_count
            FROM local_tracks
            WHERE 1=1 {} {}
            GROUP BY name
            ORDER BY name
        "#,
            source_filter, network_filter
        );

        let mut stmt = self
            .conn
            .prepare(&query)
            .map_err(|e| LibraryError::Database(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(LocalArtist {
                    name: row.get(0)?,
                    album_count: row.get(1)?,
                    track_count: row.get(2)?,
                })
            })
            .map_err(|e| LibraryError::Database(e.to_string()))?;

        let mut artists = Vec::new();
        for artist in rows {
            artists.push(artist.map_err(|e| LibraryError::Database(e.to_string()))?);
        }
        Ok(artists)
    }
}
