//! Qobuz mix detail views (DailyQ / WeeklyQ / FavQ / TopQ).
//!
//! Opened from the For You Qobuz Mixes tiles. Each mix resolves to a
//! track list (built from the data the Slint MVP can source) that the
//! MixView renders and plays:
//!   - DailyQ / WeeklyQ — `/dynamic/suggest` seeded from the local
//!     play-history track ids.
//!   - FavQ — the user's favorite tracks, shuffled.
//!   - TopQ — tracks aggregated from the user's playlists.
//!
//! (Tauri's exact mix-generation — listened-track analysis payloads,
//! playlist play-stats ranking — is approximated; the same surfaces
//! and playback result, sourced from available backend.)

mod apply;
mod item;
mod load;
mod seed;
mod select;
mod state;

pub use apply::{apply_mix, artwork_jobs, reset_mix};
pub use load::load_mix;
pub use select::{
    clear_selection, recount_selected, select_all, selected_ids, selected_play_tracks,
    set_multi_select,
};
pub use state::{current_tracks, index_of, shuffled_tracks};
