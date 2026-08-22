//! Audio settings persistence
//!
//! Stores user preferences for audio output device, exclusive mode, and DAC passthrough.
//!
//! NOTE: Tauri command wrappers remain in qbz-nix. This module contains only
//! the core types and persistence logic.
//!
//! Split by pure/IO/thread-safety-wrapper responsibility:
//! - `types` / `defaults`: the `AudioSettings` data struct and its `Default` impl.
//! - `schema` / `seed`: table creation, column migrations, and first-run seeding.
//! - `store_core` / `store_get` / `store_setters_*` / `reset`: the SQLite-backed
//!   `AudioSettingsStore`, split by CRUD concern (all `impl AudioSettingsStore`
//!   blocks for the same type, just spread across files).
//! - `state`: the thread-safe `AudioSettingsState` wrapper.

mod defaults;
mod reset;
mod schema;
mod seed;
mod state;
mod store_core;
mod store_get;
mod store_setters_device;
mod store_setters_fallback;
mod store_setters_output;
mod store_setters_playback;
mod types;

#[cfg(test)]
mod tests;

pub use state::AudioSettingsState;
pub use store_core::AudioSettingsStore;
pub use types::AudioSettings;
