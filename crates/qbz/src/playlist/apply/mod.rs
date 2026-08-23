//! Push loaded worker-thread data into `PlaylistState`, and the Qobuz-only
//! `CURRENT` track cache the play/shuffle/removal paths read back from.

mod apply_fn;
mod reset;
mod statics;

pub use apply_fn::{apply, apply_local_items, artwork_jobs};
pub use reset::reset;
pub use statics::{current_tracks, is_mixed, shuffled_tracks};
pub(super) use statics::CURRENT;
