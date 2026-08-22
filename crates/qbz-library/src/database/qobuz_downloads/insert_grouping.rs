//! Insert a Qobuz-downloaded track with full metadata and album grouping
//! (album_group_key/title, artwork_path), used by the download pipeline.

use rusqlite::params;

use crate::LibraryError;

use super::super::LibraryDatabase;

impl LibraryDatabase {
    /// Insert a Qobuz cached track with full metadata and album grouping
    /// Note: Database source field remains 'qobuz_download' for compatibility
    pub fn insert_qobuz_cached_track_with_grouping(
        &self,
        track_id: u64,
        title: &str,
        artist: &str,
        album: Option<&str>,
        album_artist: Option<&str>,
        track_number: Option<u32>,
        disc_number: Option<u32>,
        year: Option<u32>,
        duration_secs: u64,
        file_path: &str,
        album_group_key: &str,
        album_group_title: &str,
        bit_depth: Option<u32>,
        sample_rate: Option<f64>,
        artwork_path: Option<&str>,
    ) -> Result<(), LibraryError> {
        use std::time::SystemTime;

        // First, remove any existing entry for this qobuz_track_id to prevent duplicates
        let _ = self.remove_qobuz_cached_track(track_id);

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
                album_group_key, album_group_title,
                artwork_path,
                source, qobuz_track_id
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, 'qobuz_download', ?20)
            "#,
            params![
                file_path,
                title,
                artist,
                album.unwrap_or("Unknown Album"),
                album_artist.unwrap_or(artist),
                track_number.map(|v| v as i64),
                disc_number.map(|v| v as i64),
                year.map(|v| v as i64),
                duration_secs as i64,
                "flac",
                bit_depth.map(|v| v as i64),
                sample_rate.unwrap_or(44100.0),
                2, // Assume stereo
                file_size_bytes,
                now,
                now,
                album_group_key,
                album_group_title,
                artwork_path,
                track_id as i64,
            ],
        )
        .map_err(|e| LibraryError::Database(format!("Failed to insert Qobuz cached track: {}", e)))?;
        Ok(())
    }
}
