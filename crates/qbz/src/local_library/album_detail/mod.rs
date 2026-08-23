//! Album detail: local albums reuse the shared `AlbumPageView` + `AlbumState`:
//! we just populate the state from the album's local tracks and flag
//! `is-local` so the media-action dispatcher routes play to local playback.

mod apply;
mod load;
mod resolve;
mod state;

pub use apply::apply_album_version;
pub use load::{open_local_album, search_album};
pub use resolve::{album_version_dir, current_album_disc_tracks, current_album_version_tracks};
