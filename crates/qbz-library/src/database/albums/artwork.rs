use rusqlite::params;

use crate::LibraryError;

use super::super::LibraryDatabase;

impl LibraryDatabase {
    pub fn get_albums_without_artwork(
        &self,
    ) -> Result<Vec<(String, String, String)>, LibraryError> {
        let mut stmt = self
            .conn
            .prepare(
                r#"
            SELECT
                group_key,
                MIN(title) as title,
                CASE
                    WHEN COUNT(DISTINCT artist) > 1 THEN 'Various Artists'
                    ELSE MIN(artist)
                END as artist
            FROM (
                SELECT
                    COALESCE(album_group_key, album || '|' || COALESCE(album_artist, artist)) as group_key,
                    COALESCE(album_group_title, album) as title,
                    COALESCE(album_artist, artist) as artist,
                    artwork_path
                FROM local_tracks
                WHERE artwork_path IS NULL OR artwork_path = ''
            )
            GROUP BY group_key
            ORDER BY artist, title
        "#,
            )
            .map_err(|e| LibraryError::Database(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(|e| LibraryError::Database(e.to_string()))?;

        let mut albums = Vec::new();
        for album in rows {
            albums.push(album.map_err(|e| LibraryError::Database(e.to_string()))?);
        }
        Ok(albums)
    }

    /// Update artwork path for all tracks in an album
    pub fn update_album_artwork(
        &self,
        album: &str,
        artist: &str,
        artwork_path: &str,
    ) -> Result<(), LibraryError> {
        self.conn
            .execute(
                r#"
            UPDATE local_tracks
            SET artwork_path = ?
            WHERE album = ? AND COALESCE(album_artist, artist) = ?
        "#,
                params![artwork_path, album, artist],
            )
            .map_err(|e| LibraryError::Database(e.to_string()))?;
        Ok(())
    }

    /// Update artwork path for all tracks in an album group.
    ///
    /// **Deprecated**: this was used inside the scan loop to backfill
    /// artwork across tracks in the same group, but it pisses every
    /// track's individual artwork in the process — destroying unique
    /// per-track embedded covers. Per-track artwork is now resolved
    /// individually at scan time. Kept compilable for any caller that
    /// might still exist; do not introduce new callers.
    #[deprecated(note = "Was destructive in scan loop; per-track artwork is resolved during scan instead")]
    pub fn update_album_group_artwork(
        &self,
        group_key: &str,
        artwork_path: &str,
    ) -> Result<(), LibraryError> {
        self.conn
            .execute(
                r#"
            UPDATE local_tracks
            SET artwork_path = ?
            WHERE COALESCE(album_group_key, album || '|' || COALESCE(album_artist, artist)) = ?
        "#,
                params![artwork_path, group_key],
            )
            .map_err(|e| LibraryError::Database(e.to_string()))?;
        Ok(())
    }
}
