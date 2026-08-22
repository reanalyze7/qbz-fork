use rusqlite::params;

use crate::{LibraryError, LocalTrack};

use super::super::helpers::escape_like_pattern;
use super::super::LibraryDatabase;

impl LibraryDatabase {
    /// List ALL tracks recursively under a folder (every descendant, at
    /// any depth). Mirrors the source filter and LIKE-escape strategy
    /// from [`Self::list_folder_tracks`] but does NOT require the
    /// `file_path` to live directly inside `folder_path` — every row
    /// matching `file_path LIKE folder_path || '/%'` is included.
    ///
    /// Used by the tree-mode multi-select to populate the union of
    /// `selectedTrackIds` when the user ticks a folder-row checkbox.
    /// Returns the full track records (not just IDs) so the frontend
    /// can build queue items for "Play Next" / "Add to Queue" without
    /// a second round-trip.
    ///
    /// Ordering: by `file_path` ASC. This produces a stable, on-disk
    /// reading order for cross-album / cross-disc subtrees, matching
    /// the way `handlePlayRecursive` sorts before queuing.
    pub fn list_folder_tracks_recursive(
        &self,
        folder_path: &str,
        exclude_network_folders: bool,
    ) -> Result<Vec<LocalTrack>, LibraryError> {
        let escaped_prefix = escape_like_pattern(folder_path);

        // Network filter mirrors the flat-mode `v2_library_search` /
        // `get_albums_with_full_filter` predicate so the recursive
        // multi-select boundary matches what the tree rail and the
        // playback path see.
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
               AND COALESCE(source, 'user') = 'user' \
               {network_filter} \
             ORDER BY file_path ASC",
            cols = Self::TRACK_COLUMNS,
            network_filter = network_filter,
        );

        let mut stmt = self
            .conn
            .prepare(&sql)
            .map_err(|e| LibraryError::Database(e.to_string()))?;

        let rows = stmt
            .query_map(params![escaped_prefix], |row| Self::row_to_track(row))
            .map_err(|e| LibraryError::Database(e.to_string()))?;

        let mut tracks = Vec::new();
        for track in rows {
            tracks.push(track.map_err(|e| LibraryError::Database(e.to_string()))?);
        }
        Ok(tracks)
    }

    /// Lightweight `COUNT(*)` of every user track whose `file_path` lives
    /// recursively under `folder_path`. Used by the tree-mode rail to
    /// populate the recursive descendant count on top-level scan-root
    /// rows (which are synthesized client-side and don't go through
    /// [`Self::list_folder_children`], so they don't carry their own
    /// precomputed `track_count_under`).
    ///
    /// Source filter (`COALESCE(source, 'user') = 'user'`) and the
    /// optional network-folder NOT EXISTS predicate match the listing
    /// primitives byte-for-byte so the count, the rail visibility, and
    /// recursive playback all agree on the same boundary.
    pub fn count_folder_tracks_recursive(
        &self,
        folder_path: &str,
        exclude_network_folders: bool,
    ) -> Result<u32, LibraryError> {
        let escaped_prefix = escape_like_pattern(folder_path);

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
            "SELECT COUNT(*) FROM local_tracks \
             WHERE file_path LIKE ?1 || '/%' ESCAPE '\\' \
               AND COALESCE(source, 'user') = 'user' \
               {network_filter}",
            network_filter = network_filter,
        );

        let count: i64 = self
            .conn
            .query_row(&sql, params![escaped_prefix], |row| row.get(0))
            .map_err(|e| LibraryError::Database(e.to_string()))?;
        Ok(count.try_into().unwrap_or(0))
    }
}
