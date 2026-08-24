//! Slint offline-cache controller.
//!
//! Triggers caching (single / batch) and removal, and drives the per-row
//! offline status + the unlock padlock through a `CacheEventSink` that
//! pushes updates onto the visible track models (mirrors the favorite-state
//! machinery: `set_row_cache_status` / `set_row_unlocking` in `main.rs`).
//!
//! The heavy lifting (download pipeline, CMAF store, vault) lives in the
//! shared `qbz-offline-cache` crate; this is the thin Slint orchestration.

mod cache_batch;
mod cache_bulk;
mod cache_single;
mod clear;
mod ids;
mod info;
mod redownload;
mod remove_album;
mod remove_track;
mod sink;


pub use cache_batch::cache_tracks;
pub use cache_bulk::{cache_album, cache_playlist};
pub use cache_single::cache_track;
pub use clear::{clear_all, open_folder};
pub use ids::{cached_ids_set, is_cached, load_cached_ids};
pub use redownload::{redownload_album, redownload_track};
pub use remove_album::remove_album;
pub use remove_track::{refresh_cached, remove_cached};
pub use sink::row_sink;
