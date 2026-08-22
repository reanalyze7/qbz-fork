use rusqlite::{params, OptionalExtension};

use crate::LibraryError;

use super::super::LibraryDatabase;
use super::super::TrackMetadataUpdateFull;

impl LibraryDatabase {
    pub fn update_tracks_metadata_by_id(
        &mut self,
        updates: &[TrackMetadataUpdateFull],
    ) -> Result<(), LibraryError> {
        let tx = self
            .conn
            .transaction()
            .map_err(|e| LibraryError::Database(e.to_string()))?;

        {
            let mut stmt = tx
                .prepare(
                    r#"
                    UPDATE local_tracks
                    SET
                        title = ?1,
                        artist = ?2,
                        album = ?3,
                        album_artist = ?4,
                        album_group_title = ?5,
                        track_number = ?6,
                        disc_number = ?7,
                        year = ?8,
                        genre = ?9,
                        catalog_number = ?10
                    WHERE id = ?11
                    "#,
                )
                .map_err(|e| LibraryError::Database(e.to_string()))?;

            for update in updates {
                stmt.execute(params![
                    update.title.trim(),
                    update.artist.trim(),
                    update.album.trim(),
                    update.album_artist.as_ref().map(|s| s.trim().to_string()),
                    update.album_group_title.trim(),
                    update.track_number,
                    update.disc_number,
                    update.year,
                    update.genre.as_ref().map(|s| s.trim().to_string()),
                    update.catalog_number.as_ref().map(|s| s.trim().to_string()),
                    update.id
                ])
                .map_err(|e| LibraryError::Database(e.to_string()))?;
            }
        }

        tx.commit()
            .map_err(|e| LibraryError::Database(e.to_string()))?;
        Ok(())
    }

    pub fn find_album_group_key(
        &self,
        album: &str,
        artist: &str,
    ) -> Result<Option<String>, LibraryError> {
        self.conn
            .query_row(
                r#"
            SELECT COALESCE(album_group_key, album || '|' || COALESCE(album_artist, artist))
            FROM local_tracks
            WHERE album = ? AND COALESCE(album_artist, artist) = ?
            LIMIT 1
        "#,
                params![album, artist],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| LibraryError::Database(e.to_string()))
    }
}
