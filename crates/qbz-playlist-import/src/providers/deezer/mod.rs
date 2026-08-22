//! Deezer playlist import

mod detect;
mod fetch;

pub use detect::{detect_resource, parse_playlist_id};
pub use fetch::fetch_playlist;
