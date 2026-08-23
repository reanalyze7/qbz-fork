//! Slint conversion + state push: worker-thread `HomeData` -> `HomeState`.

mod apply;
mod items;
mod sections;

pub use apply::{apply_home, apply_recent_rails};
pub(crate) use items::{card_to_item, playlist_to_item};
pub use sections::playlist_artwork_jobs;
