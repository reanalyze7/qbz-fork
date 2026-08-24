//! Polish carousels: "more from artist", "listening suggestions", and the
//! Last.fm "similar albums" row underneath.

mod more_from_artist;
mod suggestions;

pub use more_from_artist::{apply_more_from_artist, load_more_from_artist};
pub use suggestions::{apply_lastfm_suggestions, apply_suggestions, load_suggestions};
