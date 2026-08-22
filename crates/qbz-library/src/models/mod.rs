//! Data models for local library
//!
//! Split by domain: tracks, albums/artists, scan progress, album
//! settings, folder tree, and artist images. Every public item is
//! re-exported here so `qbz_library::models::*` (and the crate root's
//! `pub use models::*;`) resolves identically to the pre-split layout.

mod album;
mod album_settings;
mod artist_image;
mod audio_format;
mod folder_tree;
mod playlist_track;
mod scan;
mod track;

pub use album::*;
pub use album_settings::*;
pub use artist_image::*;
pub use audio_format::*;
pub use folder_tree::*;
pub use playlist_track::*;
pub use scan::*;
pub use track::*;
