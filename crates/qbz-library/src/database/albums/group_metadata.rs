use rusqlite::params;

use crate::LibraryError;

use super::super::LibraryDatabase;
use super::super::AlbumTrackUpdate;

impl LibraryDatabase {
    pub fn update_album_group_metadata(
        &mut self,
        group_key: &str,
        album_title: &str,
        album_artist: &str,
        year: Option<u32>,
        genre: Option<&str>,
        catalog_number: Option<&str>,
        track_artist_match: Option<&str>,
        track_updates: &[AlbumTrackUpdate],
    ) -> Result<(), LibraryError> {
        let tx = self
            .conn
            .transaction()
            .map_err(|e| LibraryError::Database(e.to_string()))?;

        let normalized_album_artist = {
            let trimmed = album_artist.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        };

        tx.execute(
            r#"
            UPDATE local_tracks
            SET
                album = ?1,
                album_group_title = ?2,
                album_artist = ?3,
                year = ?4,
                genre = ?5,
                catalog_number = ?6
            WHERE COALESCE(album_group_key, album || '|' || COALESCE(album_artist, artist)) = ?7
            "#,
            params![
                album_title.trim(),
                album_title.trim(),
                normalized_album_artist,
                year,
                genre.map(|s| s.trim()).filter(|s| !s.is_empty()),
                catalog_number.map(|s| s.trim()).filter(|s| !s.is_empty()),
                group_key
            ],
        )
        .map_err(|e| LibraryError::Database(e.to_string()))?;

        if let Some(match_artist) = track_artist_match {
            let match_trim = match_artist.trim();
            if !match_trim.is_empty() && !album_artist.trim().is_empty() {
                tx.execute(
                    r#"
                    UPDATE local_tracks
                    SET artist = ?1
                    WHERE COALESCE(album_group_key, album || '|' || COALESCE(album_artist, artist)) = ?2
                      AND artist = ?3
                    "#,
                    params![album_artist.trim(), group_key, match_trim],
                )
                .map_err(|e| LibraryError::Database(e.to_string()))?;
            }
        }

        {
            let mut stmt = tx
                .prepare("UPDATE local_tracks SET title = ?1, disc_number = ?2, track_number = ?3 WHERE id = ?4")
                .map_err(|e| LibraryError::Database(e.to_string()))?;

            for update in track_updates {
                stmt.execute(params![
                    update.title.trim(),
                    update.disc_number,
                    update.track_number,
                    update.id
                ])
                .map_err(|e| LibraryError::Database(e.to_string()))?;
            }
        }

        tx.commit()
            .map_err(|e| LibraryError::Database(e.to_string()))?;
        Ok(())
    }
}
