//! Playlist folders — local-only organization stored in library.db
//! (shared with the Tauri app). Folders are flat (no nesting); a
//! playlist belongs to at most one folder via
//! `playlist_settings.folder_id`. All ops are blocking (they open the
//! DB), so async callers wrap them in `tokio::task::spawn_blocking`.

mod manager;
mod sidebar;

pub use manager::*;
pub use sidebar::*;
