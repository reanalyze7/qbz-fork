//! Simple direct insert of a Qobuz-downloaded track (no album grouping).

use rusqlite::params;

use crate::LibraryError;

use super::super::LibraryDatabase;

impl LibraryDatabase {
    /// Insert a Qobuz cached track into the library
    /// Note: Database source field remains 'qobuz_download' for compatibility
    pub fn insert_qobuz_cached_track_direct(
        &self,
        track_id: u64,
        title: &str,
        artist: &str,
        album: Option<&str>,
        duration_secs: u64,
        file_path: &str,
        bit_depth: Option<u32>,
        sample_rate: Option<f64>,
        track_number: Option<u32>,
        disc_number: Option<u32>,
    ) -> Result<(), LibraryError> {
        use std::time::SystemTime;

        // Get file size if file exists
        let file_size_bytes = std::fs::metadata(file_path)
            .map(|m| m.len() as i64)
            .unwrap_or(0);

        // Get current timestamp
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        self.conn.execute(
            r#"
            INSERT INTO local_tracks (
                file_path, title, artist, album, album_artist,
                track_number, disc_number, year, duration_secs,
                format, bit_depth, sample_rate, channels,
                file_size_bytes, last_modified, indexed_at,
                source, qobuz_track_id
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, 'qobuz_download', ?17)
            "#,
            params![
                file_path,
                title,
                artist,
                album.unwrap_or("Unknown Album"),
                artist, // Use artist as album_artist for proper grouping
                track_number.map(|v| v as i64),
                disc_number.map(|v| v as i64),
                None::<u32>, // year
                duration_secs as i64,
                "flac", // Default format for downloads
                bit_depth.map(|v| v as i64),
                sample_rate.unwrap_or(44100.0),
                2, // Assume stereo
                file_size_bytes,
                now,
                now,
                track_id as i64,
            ],
        )
        .map_err(|e| LibraryError::Database(format!("Failed to insert Qobuz cached track: {}", e)))?;
        Ok(())
    }
}
