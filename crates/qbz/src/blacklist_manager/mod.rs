//! Blacklist Manager controller — the Rust side of the Slint port of Tauri's
//! BlacklistManagerView (Task 11). Loads the per-user blacklist into
//! `BlacklistState`, applies search-as-you-type filtering controller-side, and
//! runs the toggle / remove / clear mutations against the
//! `crate::artist_blacklist` wrapper (the same fail-open singleton the artist
//! toggle in T9 mutates). A third "Recommendations" tab (active-tab 2) lists
//! the reco-SCOPED "Not interested" dismissals from `crate::reco_dismiss` —
//! NOT the blacklist — with a per-row undo.
//!
//! Mirrors `crate::offline_manager`'s shape: a `load` entry point invoked on
//! navigation + an action set wired in `main.rs`. There is no change-notify on
//! the blacklist store (the fav_cache pattern), so the manager reloads on every
//! `open` — a mutation from elsewhere (the T9 artist toggle) is reflected the
//! next time the manager is opened, and the manager's own mutations re-push the
//! filtered list in place.
//!
//! Search filter (Tauri parity §7): trim the query; empty → the full list;
//! else a case-insensitive substring match on `artist_name` ONLY (notes are not
//! searched), preserving the backend's name-sorted order. `count` always
//! carries the FULL list length so the view can tell "empty blacklist"
//! (count==0) from "no search results" (count>0, filtered list empty).
//!
//! Date: `added_at` is unix SECONDS; formatted controller-side to "MMM D, YYYY"
//! (English; the Slint build has no gettext for Rust strings — matches T9
//! toasts being `format!` English).

mod album_actions;
mod artist_actions;
mod build;
mod build_album;
mod dismiss_actions;
mod state;

pub use album_actions::{block_album, clear_all_albums, remove_album, set_tab};
pub use artist_actions::{clear_all, remove, search_changed, toggle_enabled};
pub use build::load;
pub use dismiss_actions::remove_dismissed;
pub use state::set_image_cache;
