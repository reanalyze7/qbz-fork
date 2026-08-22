use rusqlite::params;

use crate::{LibraryError, LocalTrack};

use super::super::LibraryDatabase;

impl LibraryDatabase {
    /// Check if a file path is already registered as a Qobuz cached track
    /// Returns true if the file exists with source = 'qobuz_download' (legacy name kept for DB compatibility)
    pub fn is_qobuz_cached_track_by_path(&self, file_path: &str) -> Result<bool, LibraryError> {
        let count: i64 = self.conn
            .query_row(
                "SELECT COUNT(*) FROM local_tracks WHERE file_path = ?1 AND source = 'qobuz_download'",
                params![file_path],
                |row| row.get(0)
            )
            .map_err(|e| LibraryError::Database(e.to_string()))?;
        Ok(count > 0)
    }

    /// Insert or update a track (skips if file is already a Qobuz cached track)
    pub fn insert_track(&self, track: &LocalTrack) -> Result<i64, LibraryError> {
        // Don't overwrite Qobuz cached tracks with scanned data
        if self.is_qobuz_cached_track_by_path(&track.file_path)? {
            log::debug!(
                "Skipping track insert - already exists as Qobuz cached track: {}",
                track.file_path
            );
            // Return the existing ID
            return self
                .conn
                .query_row(
                    "SELECT id FROM local_tracks WHERE file_path = ?1",
                    params![track.file_path],
                    |row| row.get(0),
                )
                .map_err(|e| LibraryError::Database(e.to_string()));
        }

        // Detect if this file is a Qobuz purchased download
        let is_purchase: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM downloaded_purchases WHERE file_path = ?1",
                params![track.file_path],
                |row| row.get::<_, i64>(0),
            )
            .unwrap_or(0)
            > 0;

        let source = if is_purchase {
            "qobuz_purchase"
        } else {
            "user"
        };

        // Detect whether the audio file sits on a network-backed
        // filesystem. Done per-insert instead of per-scan-start because
        // mount topology can change between folder scans; the cost is
        // negligible (one /proc/mounts read, cached by the kernel
        // page cache).
        let is_network_mount = crate::mount_info::is_network_path(
            std::path::Path::new(&track.file_path),
        );

        self.conn
            .execute(
                r#"INSERT OR REPLACE INTO local_tracks
               (file_path, title, artist, album, album_artist, track_number,
                disc_number, year, genre, catalog_number, duration_secs, format, bit_depth,
                sample_rate, channels, file_size_bytes, cue_file_path,
                cue_start_secs, cue_end_secs, artwork_path, last_modified, indexed_at,
                album_group_key, album_group_title, source, is_network_mount)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
                params![
                    track.file_path,
                    track.title,
                    track.artist,
                    track.album,
                    track.album_artist,
                    track.track_number,
                    track.disc_number,
                    track.year,
                    track.genre,
                    track.catalog_number,
                    track.duration_secs,
                    track.format.to_string(),
                    track.bit_depth,
                    track.sample_rate,
                    track.channels,
                    track.file_size_bytes,
                    track.cue_file_path,
                    track.cue_start_secs,
                    track.cue_end_secs,
                    track.artwork_path,
                    track.last_modified,
                    track.indexed_at,
                    track.album_group_key,
                    track.album_group_title,
                    source,
                    is_network_mount as i64,
                ],
            )
            .map_err(|e| LibraryError::Database(e.to_string()))?;

        Ok(self.conn.last_insert_rowid())
    }
}
