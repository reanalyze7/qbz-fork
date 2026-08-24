//! Album detail controller.
//!
//! Fetches a full album through `QbzCore`, maps it to plain (Send) data
//! on the worker thread, and applies it to the `AlbumState` global on the
//! Slint event loop.

use std::cell::RefCell;

use qbz_models::Track;

use crate::TrackItem;

mod apply;
mod carousels;
mod data;
mod map;
mod reset;
mod selection;
#[cfg(test)]
mod tests;

pub use apply::{apply_album, apply_artwork};
pub use carousels::{
    apply_lastfm_suggestions, apply_more_from_artist, apply_suggestions, load_more_from_artist,
    load_suggestions,
};
pub use data::TrackData;
pub use map::load_album;
pub use reset::{filter_tracks, reset_album};
pub use selection::{
    clear_selection, disc_play_tracks, recount_selected, select_all, selected_ids,
    selected_play_tracks, set_multi_select,
};

thread_local! {
    /// The current album's full, unfiltered track list — kept so the
    /// track search can filter against it without a re-fetch. UI thread
    /// only, hence `thread_local`.
    static FULL_TRACKS: RefCell<Vec<TrackItem>> = RefCell::new(Vec::new());
    /// The current album's RAW catalog tracks (qbz_models::Track), kept so
    /// the multi-select bulk actions (enqueue / cache) can resolve the
    /// selected rows to full Track objects without a re-fetch — mirrors the
    /// `play` Vec the favorites tab keeps. UI thread only.
    static PLAY_TRACKS: RefCell<Vec<Track>> = RefCell::new(Vec::new());
}
