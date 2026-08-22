//! Read-only lookups over cached artist images and canonical names.

use rusqlite::{params, OptionalExtension};

use crate::LibraryError;

use super::super::LibraryDatabase;

impl LibraryDatabase {
    /// Get cached artist image
    pub fn get_artist_image(
        &self,
        artist_name: &str,
    ) -> Result<Option<crate::ArtistImageInfo>, LibraryError> {
        let result = self.conn.query_row(
            "SELECT artist_name, image_url, source, custom_image_path, canonical_name FROM artist_images WHERE artist_name = ?1",
            params![artist_name],
            |row| {
                Ok(crate::ArtistImageInfo {
                    artist_name: row.get(0)?,
                    image_url: row.get(1)?,
                    source: row.get(2)?,
                    custom_image_path: row.get(3)?,
                    canonical_name: row.get(4)?,
                })
            }
        ).optional()
        .map_err(|e| LibraryError::Database(format!("Failed to get artist image: {}", e)))?;
        Ok(result)
    }

    /// Get all custom artist images (for bulk lookup)
    pub fn get_all_custom_artist_images(
        &self,
    ) -> Result<std::collections::HashMap<String, String>, LibraryError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT artist_name, custom_image_path FROM artist_images WHERE custom_image_path IS NOT NULL",
            )
            .map_err(|e| LibraryError::Database(format!("Failed to prepare query: {}", e)))?;

        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| {
                LibraryError::Database(format!("Failed to query custom artist images: {}", e))
            })?;

        let mut map = std::collections::HashMap::new();
        for row in rows {
            if let Ok((artist_name, custom_image_path)) = row {
                map.insert(artist_name, custom_image_path);
            }
        }
        Ok(map)
    }

    /// Bulk-load every cached artist image (custom path preferred, else the
    /// fetched Qobuz URL) keyed by artist_name. Lets a UI seed the rail with
    /// previously-fetched portraits on revisit without re-hitting Qobuz.
    /// (The Tauri batch command `library_get_artist_images` was never
    /// registered; this is the corrected one-pass reader.)
    pub fn get_all_artist_image_urls(
        &self,
    ) -> Result<std::collections::HashMap<String, String>, LibraryError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT artist_name, custom_image_path, image_url FROM artist_images \
                 WHERE custom_image_path IS NOT NULL OR image_url IS NOT NULL",
            )
            .map_err(|e| LibraryError::Database(format!("Failed to prepare query: {}", e)))?;

        let rows = stmt
            .query_map([], |row| {
                let custom: Option<String> = row.get(1)?;
                let url: Option<String> = row.get(2)?;
                Ok((row.get::<_, String>(0)?, custom.or(url)))
            })
            .map_err(|e| {
                LibraryError::Database(format!("Failed to query artist images: {}", e))
            })?;

        let mut map = std::collections::HashMap::new();
        for row in rows.flatten() {
            let (name, maybe_path) = row;
            if let Some(path) = maybe_path {
                map.insert(name, path);
            }
        }
        Ok(map)
    }

    /// Get all canonical artist names mapping (for bulk lookup)
    pub fn get_all_canonical_names(
        &self,
    ) -> Result<std::collections::HashMap<String, String>, LibraryError> {
        let mut stmt = self.conn.prepare(
            "SELECT artist_name, canonical_name FROM artist_images WHERE canonical_name IS NOT NULL"
        ).map_err(|e| LibraryError::Database(format!("Failed to prepare query: {}", e)))?;

        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| {
                LibraryError::Database(format!("Failed to query canonical names: {}", e))
            })?;

        let mut map = std::collections::HashMap::new();
        for row in rows {
            if let Ok((artist_name, canonical_name)) = row {
                map.insert(artist_name, canonical_name);
            }
        }
        Ok(map)
    }
}
