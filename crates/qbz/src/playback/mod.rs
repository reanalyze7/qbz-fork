//! Playback / queue controller.
//!
//! Owns the orchestration between the UI and `QbzCore`'s player + queue.
//! Albums and tracks are turned into a `Vec<QueueTrack>`, handed to the
//! core's `QueueManager`, and then played audibly through
//! `Player::play_track` (the self-contained "fetch URL → download → play"
//! path — the protected bit-perfect audio backend is untouched).
//!
//! There is no event stream from the player, so a `tokio` poll task reads
//! `Player::get_playback_event()` a few times a second and pushes the
//! values onto the `NowPlayingState` global. The same task drives
//! auto-advance when a track ends.
//!
//! Split into one module per responsibility cluster (see
//! `refactor-plans/crates__qbz__src__playback.rs.md`): shared statics live
//! in `state.rs`, the poll loop's phases live under `poll/`, and every
//! other cluster keeps its own file/subdirectory. `mod.rs` re-exports the
//! full original flat API so no caller's `crate::playback::X` path changes.

use std::sync::Arc;

use qbz_app::shell::AppRuntime;

use crate::adapter::SlintAdapter;

mod advance;
mod context_play;
mod engine;
mod enqueue;
mod loading;
mod local;
mod meta;
mod quality;
mod queue_build;
mod queue_context;
mod recent_blacklist;
mod poll;
mod seek_display;
mod state;
#[cfg(test)]
mod tests;
mod transport;
mod transport_mode;
mod transport_volume;

type Runtime = Arc<AppRuntime<SlintAdapter>>;

pub use context_play::{
    enqueue_artist_top_selected, play_album, play_album_from, play_artist, play_artist_top_from,
    play_artist_top_shuffled, play_artist_top_tracks, play_label_top_shuffled,
};
pub use engine::after_track_change;
pub use enqueue::{
    enqueue_album, enqueue_album_next, enqueue_local_tracks, enqueue_playlist, enqueue_queue_tracks,
    enqueue_track, enqueue_track_ids, enqueue_tracks, play_album_shuffled, play_playlist,
    play_track_next,
};
pub(crate) use local::local_queue_track;
pub use local::{
    ephemeral_enqueue, ephemeral_play, ephemeral_play_or_prompt, fill_missing_covers, play_ephemeral_album,
    play_ephemeral_all, play_ephemeral_track, play_local_album, play_local_folder_recursive,
    play_local_folder_tracks_from, play_local_tracks, play_local_tracks_from, wipe_ephemeral_if_playing,
};
pub(crate) use meta::refresh_now_playing_meta;
pub(crate) use meta::{classify_limit_cause, delivered_tier_str, stream_downgraded};
pub use meta::NOTIFICATIONS_ENABLED;
pub(crate) use quality::{local_playback_quality, playback_quality};
pub(crate) use queue_context::{make_queue_track, stamp_queue_context};
pub use queue_context::set_now_playing_context;
pub use queue_build::{play_track_in_context, play_track_now, play_tracks, play_tracks_ctx};
pub use poll::start_poll_loop;
pub(crate) use seek_display::seed_seek_display;
pub use state::set_queue_controller;
pub(crate) use state::refresh_sidebar;
pub use transport::{next, previous, seek, toggle_play_pause};
pub use transport_mode::{cycle_repeat, toggle_shuffle};
pub use transport_volume::{set_volume, toggle_mute};
