//! Tracks tab: server-paginated flat list (the perf path that avoids the
//! documented ~16K freeze): each page is a `search_with_filter_page` query,
//! appended on scroll. Track artwork is off by default (Tauri's perf
//! default), so there are no per-row artwork jobs here. Group-by /
//! multi-select / per-row playback land with the source-aware playback
//! slice.

mod derive;
pub(crate) mod load;
pub(crate) mod map;
mod select;

pub use derive::*;
pub use load::{
    ensure_tracks_loaded, load_more_tracks, reload_tracks, tracks_current_snapshot,
};
pub use select::*;
