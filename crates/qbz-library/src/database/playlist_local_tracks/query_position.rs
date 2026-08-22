use rusqlite::params;

use crate::{LibraryError, LocalTrack};

use super::super::LibraryDatabase;

impl LibraryDatabase {
    /// Get all local tracks in a playlist with their positions (for mixed ordering)
    pub fn get_playlist_local_tracks_with_position(
        &self,
        qobuz_playlist_id: u64,
    ) -> Result<Vec<crate::PlaylistLocalTrack>, LibraryError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT t.id, t.file_path, t.title, t.artist, t.album, t.album_artist,
                    t.album_group_key, t.album_group_title, t.track_number, t.disc_number,
                    t.year, t.genre, t.duration_secs, t.format, t.bit_depth, t.sample_rate,
                    t.channels, t.file_size_bytes, t.cue_file_path, t.cue_start_secs,
                    t.cue_end_secs, t.artwork_path, t.last_modified, t.indexed_at, t.source,
                    t.qobuz_track_id, t.is_network_mount, plt.position
             FROM playlist_local_tracks plt
             JOIN local_tracks t ON plt.local_track_id = t.id
             WHERE plt.qobuz_playlist_id = ?1
             ORDER BY plt.position ASC",
            )
            .map_err(|e| LibraryError::Database(format!("Failed to prepare statement: {}", e)))?;

        let tracks = stmt
            .query_map(params![qobuz_playlist_id as i64], |row| {
                Ok(crate::PlaylistLocalTrack {
                    track: LocalTrack {
                        id: row.get(0)?,
                        file_path: row.get(1)?,
                        title: row.get(2)?,
                        artist: row.get(3)?,
                        album: row.get(4)?,
                        album_artist: row.get(5)?,
                        album_group_key: row.get::<_, Option<String>>(6)?.unwrap_or_default(),
                        album_group_title: row.get::<_, Option<String>>(7)?.unwrap_or_default(),
                        track_number: row.get(8)?,
                        disc_number: row.get(9)?,
                        year: row.get(10)?,
                        genre: row.get(11)?,
                        catalog_number: None,
                        duration_secs: row.get(12)?,
                        format: Self::parse_format(&row.get::<_, String>(13)?),
                        bit_depth: row.get(14)?,
                        sample_rate: row.get::<_, f64>(15)?,
                        channels: row.get(16)?,
                        file_size_bytes: row.get(17)?,
                        cue_file_path: row.get(18)?,
                        cue_start_secs: row.get(19)?,
                        cue_end_secs: row.get(20)?,
                        artwork_path: row.get(21)?,
                        last_modified: row.get(22)?,
                        indexed_at: row.get(23)?,
                        source: row.get(24)?,
                        qobuz_track_id: row.get(25)?,
                        is_network_mount: row.get::<_, i64>(26)? != 0,
                    },
                    playlist_position: row.get(27)?,
                })
            })
            .map_err(|e| {
                LibraryError::Database(format!(
                    "Failed to query playlist local tracks with position: {}",
                    e
                ))
            })?;

        tracks.collect::<Result<Vec<_>, _>>().map_err(|e| {
            LibraryError::Database(format!(
                "Failed to collect playlist local tracks with position: {}",
                e
            ))
        })
    }
}
