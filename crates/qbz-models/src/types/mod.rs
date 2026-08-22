//! Core API types for QBZ
//!
//! This module contains all shared data types used across the application:
//! - Media types: Track, Album, Artist, Playlist
//! - Quality/streaming types
//! - Search and favorites types
//! - Image and metadata types
//!
//! Split by domain into submodules (mirrors the original file's banner
//! comments); every submodule is re-exported here so callers keep using
//! `qbz_models::types::Track` / `qbz_models::Track` exactly as before.

mod album;
mod artist;
mod artist_page;
mod artist_page_release;
mod artist_page_track;
mod cmaf;
mod discover;
mod discover_album;
mod external_stream;
mod genre;
mod image;
mod label;
mod playlist;
mod quality;
mod search;
mod session;
mod stream;
mod track;

pub use album::*;
pub use artist::*;
pub use artist_page::*;
pub use artist_page_release::*;
pub use artist_page_track::*;
pub use cmaf::*;
pub use discover::*;
pub use discover_album::*;
pub use external_stream::*;
pub use genre::*;
pub use image::*;
pub use label::*;
pub use playlist::*;
pub use quality::*;
pub use search::*;
pub use session::*;
pub use stream::*;
pub use track::*;
