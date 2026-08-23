//! Local file & ephemeral playback: local-library entry points plus the
//! drag-and-drop "ephemeral" folder playback paths.

mod album;
mod dsd;
mod ephemeral;
mod ephemeral_enqueue;
pub(super) mod files;
mod folder;
pub(super) mod queue_track;

pub use album::{play_local_album, wipe_ephemeral_if_playing};
pub use ephemeral::{play_ephemeral_album, play_ephemeral_all, play_ephemeral_track, ephemeral_play};
pub use ephemeral_enqueue::{ephemeral_enqueue, ephemeral_play_or_prompt};
pub use folder::{
    play_local_folder_recursive, play_local_folder_tracks_from, play_local_tracks,
    play_local_tracks_from,
};
pub use queue_track::fill_missing_covers;
pub(crate) use queue_track::local_queue_track;
