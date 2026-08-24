//! MusicianPageView controller — loads the resolved musician + the
//! "Appears On" album grid and pushes them into `MusicianState`.
//!
//! Mirrors the Tauri MusicianPageView.svelte flow:
//!   1. Resolve the (name, role) via QbzCore::musicbrainz_resolve_musician.
//!   2. Fetch the first page of appearances
//!      (QbzCore::musicbrainz_get_musician_appearances).
//!   3. Subsequent pages come from the MusicianActions::load-more
//!      callback the view emits at the bottom of the grid.

mod load;
mod state;

pub use load::{load_more_appearances, load_musician};
pub use state::{apply_musician, append_appearances, artwork_jobs, reset_musician};

/// Page size — kept in sync with the Tauri view's ITEMS_PER_PAGE.
pub const PAGE_SIZE: u32 = 20;
