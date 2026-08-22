//! Apple Music playlist import

mod html;
mod json;
mod parse_url;
mod scrape;

pub use parse_url::{detect_resource, parse_playlist_id};
pub use scrape::fetch_playlist;
