//! Artists tab: two-column master/detail, 1:1 with Tauri's Artists tab (NOT
//! a card grid — the Svelte VirtualizedArtistGrid imports there are dead).
//! Left rail: a merged/deduped, alpha-grouped master list of compact rows
//! (round avatar + name + "N albums · M tracks"). Right pane: the selected
//! artist's albums, filtered IN PLACE from the loaded album set (no new
//! backend call). The name-merge collapses normalized-equal spellings into
//! one canonical row.
//!
//! The Qobuz background image fetch (capped/sequential in Tauri, and whose DB
//! batch path is broken there) is the remaining follow-up; this pass wires
//! the DB custom-image path + the mic placeholder.

mod derive;
pub(crate) mod images;
mod load;
mod matching;
pub(crate) mod merge;
pub(crate) mod normalize;
mod select;
pub(crate) mod state;

pub use derive::derive_artists;
pub use images::{artists_img_gen_current, set_artist_row_image};
pub use load::ensure_artists_loaded;
pub use normalize::normalize_artist;
pub use select::select_local_artist;
pub use state::{invalidate_artists, set_pending_artist};
