//! Graphics settings persistence.
//!
//! This module stores portable host rendering preferences only. Startup
//! detection, environment variable application, crash recovery, and command
//! transport stay outside `qbz-app`.

mod settings;
mod setters;
mod store;
#[cfg(test)]
mod tests;

pub use settings::GraphicsSettings;
pub use store::GraphicsSettingsStore;
