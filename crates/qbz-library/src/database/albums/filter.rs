use crate::{LibraryError, LocalAlbum};

use super::super::LibraryDatabase;

impl LibraryDatabase {
    // === Query Methods ===

    /// Get all albums with optional hidden filter
    pub fn get_albums(&self, include_hidden: bool) -> Result<Vec<LocalAlbum>, LibraryError> {
        self.get_albums_with_filter(include_hidden, true)
    }

    /// Get all albums with optional filters for hidden and Qobuz downloads
    pub fn get_albums_with_filter(
        &self,
        include_hidden: bool,
        include_qobuz_downloads: bool,
    ) -> Result<Vec<LocalAlbum>, LibraryError> {
        self.get_albums_with_full_filter(include_hidden, include_qobuz_downloads, false)
    }
}
