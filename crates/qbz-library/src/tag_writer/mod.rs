//! Direct embedded-tag writer (frontend-agnostic port of the Tauri
//! `v2_library_write_album_metadata_to_files` lofty loop). The Slint and Tauri
//! frontends both call this so the lofty logic lives in one place. Progress is
//! reported through an `on_progress` closure (no Tauri event bus); the caller
//! orchestrates the DB update + sidecar removal.

mod apply_fields;
mod match_artist;
mod types;
mod write;

pub use match_artist::compute_track_artist_match;
pub use types::{AlbumTagWrite, TrackTagWrite};
pub use write::write_album_tags_to_files;
