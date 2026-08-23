//! Custom-theme controller + persistence: wire the Settings "Custom" theme
//! option to `qbz_theme::custom` derivation.
//!
//! The user-authored [`qbz_theme::CustomThemeBase`] (12 base tokens) is
//! persisted next to the other QBZ data (`<data_dir>/qbz/custom_theme.json`)
//! and derived into a full palette by `qbz_theme::theme_from_base`. Derivation
//! is cheap (pure color math, no I/O), so every token edit re-derives and
//! re-pushes the palette live on the event loop — no debounce needed.
//!
//! This module mirrors `crate::auto_theme` for its wiring style (weak-handle
//! push through `crate::theme::push_colors`, the same path static and auto
//! themes use). Persistence mirrors `crate::ui_prefs::{load, save}`.

mod actions;
mod apply;
mod convert;
mod persist;
mod state;

pub use actions::{set_token, set_token_hex, seed_from_current, toggle_dark};
pub use apply::{apply_startup, seed_state};
pub use persist::{exists, load, load_or_seed, save};
