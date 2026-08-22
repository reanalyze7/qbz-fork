//! SQLite database for offline cache index.
//!
//! Split by domain: schema/migration (`schema`), core per-track CRUD
//! (`tracks`), sync-to-library queries (`sync`), album-scoped bulk ops
//! (`album`), aggregate stats (`stats`), and v2 CMAF-bundle fields (`cmaf`).
//! `OfflineCacheDb` is one struct defined in `schema.rs`; the other modules
//! add `impl OfflineCacheDb` blocks — valid Rust, no trait needed.

mod album;
mod cmaf;
mod schema;
mod stats;
mod sync;
mod tracks;
mod tracks_insert;
mod tracks_read;

#[cfg(test)]
mod tests;

pub use cmaf::CmafBundleRow;
pub use schema::OfflineCacheDb;

use crate::types::CachedTrackInfo;
use crate::types::OfflineCacheStatus;

/// Maps a `cached_tracks` row (with the canonical 17-column SELECT used by
/// `get_track`, `get_all_tracks`, and `get_album_tracks`) into a `CachedTrackInfo`.
///
/// SELECT must be:
/// `track_id, title, artist, album, album_id, duration_secs, file_size_bytes,
///  quality, bit_depth, sample_rate, status, progress_percent, error_message,
///  created_at, last_accessed_at, artwork_path, file_path`
pub(super) fn row_to_cached_track_info(row: &rusqlite::Row) -> rusqlite::Result<CachedTrackInfo> {
    Ok(CachedTrackInfo {
        track_id: row.get::<_, i64>(0)? as u64,
        title: row.get(1)?,
        artist: row.get(2)?,
        album: row.get(3)?,
        album_id: row.get(4)?,
        duration_secs: row.get::<_, i64>(5)? as u64,
        file_size_bytes: row.get::<_, i64>(6)? as u64,
        quality: row.get::<_, Option<String>>(7)?.unwrap_or_default(),
        bit_depth: row.get::<_, Option<i64>>(8)?.map(|v| v as u32),
        sample_rate: row.get(9)?,
        status: OfflineCacheStatus::from_str(&row.get::<_, String>(10)?),
        progress_percent: row.get::<_, i64>(11)? as u8,
        error_message: row.get(12)?,
        created_at: row.get(13)?,
        last_accessed_at: row.get(14)?,
        artwork_path: row.get::<_, Option<String>>(15)?,
        file_path: row.get(16)?,
    })
}
