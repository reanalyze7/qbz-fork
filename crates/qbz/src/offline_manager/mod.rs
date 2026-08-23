//! Offline Cache Manager controller — loads the artist→album→track rollup +
//! stats into `OfflineManagerState`. Per-item actions reuse `offline_cache::*`
//! (Slice 3); this module owns the data load, the toolbar filters (artist
//! rail / sort / show-only-failed), the album covers, and the size-limit edit.

mod filters;
mod format;
mod limit;
mod rebuild;
mod row;
mod rollup;
mod select;
mod toolbar;

pub(crate) use format::human_size;
pub use limit::set_limit;
pub use rebuild::rebuild;
pub use select::{selected_track_ids, set_all_selected, toggle_select};
pub use toolbar::{load, select_artist, set_sort, toggle_failed};

const GB: u64 = 1024 * 1024 * 1024;
