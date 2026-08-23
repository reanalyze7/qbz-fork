//! Playlist "Suggested Songs" section controller (T8).
//!
//! 1:1 port of the Svelte `PlaylistSuggestions.svelte` + the slice of
//! `PlaylistDetailView.svelte` that derives the seed artists / excludes and
//! mounts the component. All assembly logic lives in Rust (ADR-006): a
//! Rust-held pool + pagination feeds `PlaylistSuggestionsState`; the `.slint`
//! section only renders the projected rows + flags and fires the `Actions`.
//!
//! DISTINCT from the immersive `crate::suggestions` controller (different
//! surface, different engine). The backend engine is `qbz_reco` via
//! `core.generate_playlist_suggestions(...)`; the per-playlist dismiss store is
//! `crate::playlist_suggestions_dismiss` (T10).
//!
//! Pool sizing + pagination mirror the Svelte constants:
//!   VISIBLE_COUNT=6, INITIAL_POOL=30, EXPANDED_POOL=100, MAX_POOL=200,
//!   auto-expand when the filtered (available) pool falls below 12.
//! Filtering removes: dismissed ids (T10 store), excluded ids (already in the
//! playlist), suggestions that duplicate an existing track by `title|artist`,
//! and duplicates within the pool itself by the same key.

mod activate;
mod adaptive_artists;
mod add_track;
mod auto_expand;
mod dismiss_reset;
mod fetch;
mod filter_project;
mod reload;
mod session;

use qbz_app::shell::AppRuntime;

use crate::adapter::SlintAdapter;
use crate::AppWindow;

pub(super) type Runtime = std::sync::Arc<AppRuntime<SlintAdapter>>;
pub(super) type Handle = tokio::runtime::Handle;
pub(super) type Weak = slint::Weak<AppWindow>;

// --- Svelte parity constants -----------------------------------------------
pub(super) const VISIBLE_COUNT: usize = 6;
pub(super) const INITIAL_POOL: usize = 30;
pub(super) const EXPANDED_POOL: usize = 100;
pub(super) const MAX_POOL: usize = 200;
/// Auto-expand the pool once the available (filtered) tracks drop below this.
pub(super) const MIN_AVAILABLE_THRESHOLD: usize = 12;

pub use activate::{activate, refresh};
pub use add_track::{add_track, play_track};
pub use dismiss_reset::{dismiss_track, reset};
