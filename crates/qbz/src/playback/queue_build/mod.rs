//! Track/queue building: turning views' visible track lists (or a clicked
//! track id) into a `Vec<QueueTrack>` and handing it to the core.

mod from_model;
mod model_helpers;
mod play_queue;
mod track_in_context;
mod track_now;

pub(super) use model_helpers::reorder_queue_by_visible;
pub use play_queue::{play_tracks, play_tracks_ctx};
pub use track_in_context::play_track_in_context;
pub use track_now::play_track_now;
