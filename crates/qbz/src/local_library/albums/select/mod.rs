//! Albums multi-select.
//!
//! Albums are `AlbumCardItem` (not `TrackItem`), so they get their own select
//! path (not the central toggle arm). The rendered set is either the flat
//! `albums-visible` or, when grouping is on, the concatenation of the
//! `albums-grouped` sections — both are patched so a Shift-range / select-all
//! stays consistent across the duplicate copies of an album id.

mod basics;
mod favorite;
mod query;
mod toggle;


pub use favorite::toggle_album_favorite;
pub use query::{selected_album_ids, selected_albums_tracks_blocking};
pub use toggle::*;
