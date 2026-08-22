//! Maintenance operations on the offline cache: bulk delete and re-download.
//! Pure logic — no Tauri state. Callable from any future TUI or headless binary.
//!
//! Split into `removal` (DB + on-disk deletion, and the cache-limit
//! pre-flight check) and `redownload` (the pure re-download-target filter).

mod redownload;
mod removal;

pub use redownload::select_redownload_targets;
pub use removal::{check_cache_limit, remove_album_cached_tracks, AlbumRemovalReport};
