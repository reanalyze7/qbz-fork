//! Playback context model.
//!
//! A playback context describes the semantic origin of playback. It is not the
//! queue itself; it is the source boundary used by commands that need to know
//! whether the current playback came from an album, playlist, radio session,
//! search result, or another app-level source.

mod context;
mod manager;
#[cfg(test)]
mod tests;
mod types;

pub use context::PlaybackContext;
pub use manager::ContextManager;
pub use types::{ContentSource, ContextType};
