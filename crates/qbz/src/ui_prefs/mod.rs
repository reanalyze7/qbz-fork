//! Tiny JSON-backed UI preference store.
//!
//! Some settings the Tauri app exposes are not part of any domain store
//! (`AudioSettings`, `PlaybackPreferences`). Streaming Quality is one: it
//! is a pure UI/request preference. Rather than thread it into a domain
//! store, this module persists those preferences to a small JSON file
//! next to the other QBZ data (`<data_dir>/qbz/ui_prefs.json`).
//!
//! The store is intentionally minimal — read-modify-write the whole file
//! on every set. The file is tiny and writes are rare (a settings change).

mod defaults;
mod index_maps;
mod index_maps_display;
mod io;
mod model;
mod model_default;
mod quality;
#[cfg(test)]
mod tests;

pub use index_maps::*;
pub use index_maps_display::*;
pub use io::{load, save};
pub use model::UiPrefs;
pub use quality::{
    streaming_quality_for_key, streaming_quality_index, STREAMING_QUALITIES,
};
