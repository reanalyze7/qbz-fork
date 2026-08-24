//! Qobuz-domain -> row mapping layer (unit-tested).

mod album_track;
mod all;
mod artist_playlist;

pub use album_track::{map_album, map_track};
pub use all::map_search_all;
pub use artist_playlist::{map_artist, map_playlist};
