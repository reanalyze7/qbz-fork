//! Album/artist/label context resolution: fetch-and-play entry points that
//! build a fresh queue from a Qobuz container (album, artist page, label).

mod album;
mod artist_enqueue;
mod artist_fetch;
mod artist_play;
mod artist_shuffle;
mod artist_studio;
mod artist_top;

pub use album::{play_album, play_album_from};
pub use artist_enqueue::{enqueue_artist_top_selected, play_artist_top_from};
pub use artist_play::play_artist;
pub use artist_shuffle::{play_artist_top_shuffled, play_label_top_shuffled};
pub use artist_top::play_artist_top_tracks;
