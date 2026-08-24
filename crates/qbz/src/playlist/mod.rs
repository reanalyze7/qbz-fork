//! Playlist detail view controller.
//!
//! Fetches a playlist through `QbzCore`, maps it to the shared
//! TrackItem rows + header metadata, and applies it to `PlaylistState`.
//! Mirrors `mix.rs`: a cached track list backs play-all / per-track
//! play, and an artwork-jobs pass resolves the row covers + header
//! cover off-thread.

mod apply;
mod custom_artwork;
mod custom_order;
mod load;
mod multi_select;
mod removal;
mod row_item;
mod view_state;

pub use apply::{apply, apply_local_items, artwork_jobs, current_tracks, is_mixed, reset, shuffled_tracks};
pub use custom_artwork::{clear_custom_artwork, set_custom_artwork};
pub use custom_order::{
    apply_custom_order, custom_seed_keys, full_item_ids, load_or_init_custom, move_full_item,
    move_track, persist_custom, reorder_track, swap_full_items,
};
pub use load::load;
pub(crate) use load::interleave_rows;
pub use multi_select::{clear_selection, recount_selected, select_all, set_multi_select};
pub use removal::{row_for_id, selected_queue_tracks, selected_rows, split_for_removal, SelectedRow};
pub(crate) use row_item::to_item;
pub use view_state::{filter_tracks, set_sort, set_track_artwork};
