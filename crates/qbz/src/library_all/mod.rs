//! Library "All" — the mixed feed controller (webplayer /user-library/all).
//!
//! There is NO single Qobuz endpoint for the aggregated library; the webplayer
//! merges favorites + playlists client-side. We do the same: fan out
//! to the existing per-type loaders, normalize each into a `Feed` item, merge and
//! order by "date added" (approximated from each source's server order), then push
//! into `LibraryAllState`. Search / sort / source-switch filtering all run in Rust
//! (`derive`) — Slint renders the pre-computed `items-visible`.

mod apply;
mod derive;
mod feed;
mod load;

pub use apply::{apply_library_all, artwork_jobs};
pub use derive::{derive, set_sort};
pub use load::load_library_all;
