//! Portable tray preference storage.
//!
//! This module owns persisted tray preferences only. Runtime tray creation,
//! icon updates, window hiding/showing, and emitted events remain in the
//! host application layer.

mod prefs;
mod setters;
mod state;
mod store;
#[cfg(test)]
mod tests;

pub use prefs::{normalize_tray_icon_theme, TraySettings};
pub use state::TraySettingsState;
pub use store::TraySettingsStore;
