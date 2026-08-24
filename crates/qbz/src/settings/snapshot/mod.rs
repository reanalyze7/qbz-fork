//! `SettingsSnapshot`: plain, `Send` settings data built off the UI
//! thread, plus the load/apply entry points.

mod apply;
mod assemble;
mod build;
mod types;

pub use apply::apply_snapshot;
pub use build::load_snapshot;
