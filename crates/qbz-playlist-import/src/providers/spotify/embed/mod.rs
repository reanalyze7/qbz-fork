//! Spotify embed-page scraping — network I/O.

mod metadata;
mod playlist;

pub use metadata::fetch_embed_metadata;
pub use playlist::fetch_playlist;
