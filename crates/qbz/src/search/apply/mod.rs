//! Slint event-loop writers: turn plain row/search data into `SearchState` /
//! `ImmersiveState` global updates.

mod cortinilla;
mod follow;
mod items;
mod results;

pub(crate) use items::{album_item, artist_item, playlist_item, track_item};

pub use cortinilla::{
    apply_cortinilla, cortinilla_artwork_jobs,
};
pub use follow::mark_artist_followed;
pub use results::{apply_search, recompute_hi_res_filtered, reset_search};
