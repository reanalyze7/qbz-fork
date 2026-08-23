//! Session persistence store.
//!
//! The playback queue/session state is portable application state. The current
//! Tauri/Svelte shell also stores view restoration fields in the same DB table;
//! those fields are modeled here only so the existing schema can round-trip
//! unchanged during the extraction.

mod migrations;
mod model;
mod ops;
mod pragma;
mod quick_ops;
mod schema;
#[cfg(test)]
mod tests;

pub use model::{
    PersistedPlaybackSession, PersistedQueueTrack, PersistedSessionSnapshot,
    PersistedShellViewState,
};
pub use schema::SessionStore;
