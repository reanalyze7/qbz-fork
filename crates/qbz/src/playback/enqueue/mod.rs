//! Enqueue commands: thin Slint-callback-shaped wrappers around the
//! queue-building/engine clusters that append to (or insert-next into) the
//! live queue.

mod album;
mod album_next;
mod playlist;
mod queue_tracks;
mod track;
mod tracks_batch;

pub use album::{enqueue_album, play_album_shuffled};
pub use album_next::enqueue_album_next;
pub use playlist::{enqueue_playlist, play_playlist};
pub use queue_tracks::{enqueue_local_tracks, enqueue_queue_tracks};
pub use track::{enqueue_track, play_track_next};
pub use tracks_batch::{enqueue_track_ids, enqueue_tracks};
