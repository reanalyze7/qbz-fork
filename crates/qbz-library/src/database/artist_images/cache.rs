//! Writing/refreshing cached artist images.

use crate::LibraryError;

use super::super::LibraryDatabase;

impl LibraryDatabase {
    /// Cache artist image with optional canonical name
    pub fn cache_artist_image(
        &self,
        artist_name: &str,
        image_url: Option<&str>,
        source: &str,
        custom_image_path: Option<&str>,
    ) -> Result<(), LibraryError> {
        self.cache_artist_image_with_canonical(
            artist_name,
            image_url,
            source,
            custom_image_path,
            None,
        )
    }

    /// Cache artist image with canonical name from Qobuz/Discogs
    pub fn cache_artist_image_with_canonical(
        &self,
        artist_name: &str,
        image_url: Option<&str>,
        source: &str,
        custom_image_path: Option<&str>,
        canonical_name: Option<&str>,
    ) -> Result<(), LibraryError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        self.conn.execute(
            "INSERT OR REPLACE INTO artist_images
             (artist_name, image_url, source, custom_image_path, canonical_name, fetched_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![artist_name, image_url, source, custom_image_path, canonical_name, now, now],
        )
        .map_err(|e| LibraryError::Database(format!("Failed to cache artist image: {}", e)))?;
        Ok(())
    }
}
