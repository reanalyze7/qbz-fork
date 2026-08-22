//! Cheap library-wide counters — the Tracks-tab badge and the stats panel.

use crate::LibraryError;

use super::{LibraryDatabase, LibraryStats};

impl LibraryDatabase {
    /// Cheap total local-track count — the Tracks-tab badge number without
    /// materializing the (potentially 16K-row) table. Mirrors the Tracks tab
    /// filter (include_qobuz_downloads = true, no network exclusion, no search)
    /// so the badge equals the unfiltered list length.
    pub fn count_all_local_tracks(&self) -> Result<u64, LibraryError> {
        let n: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM local_tracks", [], |row| row.get(0))
            .map_err(|e| LibraryError::Database(e.to_string()))?;
        Ok(n as u64)
    }

    /// Get library statistics
    pub fn get_stats(&self, include_qobuz_downloads: bool) -> Result<LibraryStats, LibraryError> {
        let source_filter = if include_qobuz_downloads {
            ""
        } else {
            "WHERE (source IS NULL OR source != 'qobuz_download')"
        };

        let sql = format!(
            r#"
            SELECT
                COUNT(*) as track_count,
                COUNT(DISTINCT COALESCE(album_group_key, album || '|' || COALESCE(album_artist, artist))) as album_count,
                COUNT(DISTINCT COALESCE(album_artist, artist)) as artist_count,
                COALESCE(SUM(duration_secs), 0) as total_duration,
                COALESCE(SUM(file_size_bytes), 0) as total_size
            FROM local_tracks
            {}
        "#,
            source_filter
        );

        let mut stmt = self
            .conn
            .prepare(&sql)
            .map_err(|e| LibraryError::Database(e.to_string()))?;

        stmt.query_row([], |row| {
            Ok(LibraryStats {
                track_count: row.get(0)?,
                album_count: row.get(1)?,
                artist_count: row.get(2)?,
                total_duration_secs: row.get(3)?,
                total_size_bytes: row.get(4)?,
            })
        })
        .map_err(|e| LibraryError::Database(e.to_string()))
    }
}
