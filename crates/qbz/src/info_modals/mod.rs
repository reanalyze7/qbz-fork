//! Track Info + Album Info (Credits/Review) modal controllers.
//!
//! 1:1 port of Tauri's `TrackInfoModal.svelte` + `AlbumCreditsModal.svelte`.
//! Both fetch fresh data through `QbzCore` (`get_track` / `get_album`), map it
//! to plain `Send` structs on the worker thread, then apply it to the
//! `TrackInfoState` / `AlbumInfoState` globals on the Slint event loop —
//! mirroring `crate::album::navigate_album`. Role parsing / grouping /
//! localization lives in `qbz_qobuz::performers` (frontend-agnostic, ADR-006).

mod apply;
mod format;
mod map_album;
mod map_track;
mod spawn;
mod types;

pub use spawn::{load_track_info_inline, open_album_credits, open_track_info};
