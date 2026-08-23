//! Playback and small playback-adjacent UI preferences.
//!
//! `show_context_icon` is persisted here to preserve the existing settings
//! contract, but it is a portable UI preference, not playback domain logic.

mod setters;
mod state;
mod store;
#[cfg(test)]
mod tests;
mod types;

pub use state::PlaybackPreferencesState;
pub use store::PlaybackPreferencesStore;
pub use types::{AutoplayMode, PlaybackPreferences};
