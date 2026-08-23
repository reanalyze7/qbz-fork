//! Qobuz Playlists "View all" full-list controller.
//!
//! The playlist twin of `discover_browse`: opens the Qobuz Playlists rail
//! (Home / Editor's Picks) as a paginated full grid/list page backed by
//! `/discover/playlists`. Two filters are SERVER-side — the single-select
//! category tag bar (`/playlist/getTags`, localized names) and the shared
//! "discover" genre selection both re-fetch from offset 0 — while the
//! header search box filters the loaded set client-side and disables
//! load-more while active (same contract as the album browse page).
//!
//! Unlike the album endpoints there is no sub-genre narrowing here: the
//! `genre_ids` facet (`current_genre_filter()`, top-level ancestor ids) is
//! passed through as-is — Discover playlists carry no `genre.path` to
//! narrow against client-side.
//!
//! The selected tag is process state (`SELECTED_TAG`) rather than a
//! read-back of the Slint global, so the fetch tasks can read it off the
//! UI thread. It survives genre-filter re-navigations and resets only on
//! a fresh open from the rail's "View all" link (Tauri resets on mount).

mod artwork;
mod fetch;
mod filter;
mod load_more;
mod model;
mod navigate;
mod select_tag;

pub use filter::apply_filter;
pub use load_more::load_more;
pub use navigate::navigate;
pub use select_tag::select_tag;

use std::sync::Mutex;

/// Page size — mirrors discover_browse (and Tauri's limit=50).
pub(super) const PAGE_SIZE: u32 = 50;

/// The active category tag slug ("" = All). See the module docs for why
/// this lives outside the Slint global.
pub(super) static SELECTED_TAG: Mutex<String> = Mutex::new(String::new());

pub(super) fn selected_tag() -> String {
    SELECTED_TAG
        .lock()
        .map(|s| s.clone())
        .unwrap_or_default()
}
