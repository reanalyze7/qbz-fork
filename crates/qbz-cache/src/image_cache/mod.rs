//! Image cache service
//!
//! LRU disk cache for Qobuz album/artist images.
//! - Stores images keyed by MD5 hash of URL
//! - Tracks last-access time for LRU eviction
//! - Respects a configurable max size
//! - Framework-agnostic: shared by the Tauri app and the Slint shell so
//!   both read and write the same `~/.cache/qbz/images` cache.

mod access;
mod maintenance;
mod open;
mod types;

pub use types::{ImageCacheService, ImageCacheStats};
