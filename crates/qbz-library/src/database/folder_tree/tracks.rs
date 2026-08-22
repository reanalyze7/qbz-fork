use rusqlite::params;

use crate::{LibraryError, LocalTrack};

use super::super::helpers::escape_like_pattern;
use super::super::LibraryDatabase;

impl LibraryDatabase {
    /// List the direct-child tracks of a folder (NON-recursive).
    ///
    /// Returns rows from `local_tracks` whose `file_path` is exactly
    /// `folder_path + "/" + filename` — files in subfolders are
    /// excluded. Mirrors the source filter from
    /// [`Self::list_folder_children`] so Qobuz downloads do not appear.
    /// Ordering matches the canonical album-track ordering used by
    /// [`Self::get_album_tracks`]: disc, then track number, then title.
    pub fn list_folder_tracks(
        &self,
        folder_path: &str,
        exclude_network_folders: bool,
    ) -> Result<Vec<LocalTrack>, LibraryError> {
        let escaped_prefix = escape_like_pattern(folder_path);

        // See `list_folder_children` for the rationale on the network
        // filter — same EXISTS subquery so the tree rail and direct-
        // children listing reflect the same visible-track set.
        let network_filter = if exclude_network_folders {
            "AND NOT EXISTS ( \
                SELECT 1 FROM library_folders nf \
                WHERE nf.is_network = 1 \
                AND local_tracks.file_path LIKE nf.path || '%' \
            )"
        } else {
            ""
        };

        let sql = format!(
            "SELECT {cols} FROM local_tracks \
             WHERE file_path LIKE ?1 || '/%' ESCAPE '\\' \
               AND substr(file_path, length(?2) + 2) NOT LIKE '%/%' \
               AND COALESCE(source, 'user') = 'user' \
               {network_filter} \
             ORDER BY disc_number ASC NULLS LAST, \
                      track_number ASC NULLS LAST, \
                      title COLLATE NOCASE ASC",
            cols = Self::TRACK_COLUMNS,
            network_filter = network_filter,
        );

        let mut stmt = self
            .conn
            .prepare(&sql)
            .map_err(|e| LibraryError::Database(e.to_string()))?;

        // ?1 = LIKE-escaped pattern (matches paths under the folder).
        // ?2 = unescaped path used for substr arithmetic on stored
        //      file_path (which is itself unescaped).
        let rows = stmt
            .query_map(params![escaped_prefix, folder_path], |row| {
                Self::row_to_track(row)
            })
            .map_err(|e| LibraryError::Database(e.to_string()))?;

        let mut tracks = Vec::new();
        for track in rows {
            tracks.push(track.map_err(|e| LibraryError::Database(e.to_string()))?);
        }
        Ok(tracks)
    }
}
