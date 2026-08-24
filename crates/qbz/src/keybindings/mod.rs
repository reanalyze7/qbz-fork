//! Keyboard shortcuts (hotkeys) — Rust port of the Tauri `keybindingsStore`.
//!
//! Mirrors the Tauri model 1:1: the same 26 actions, the same default
//! shortcuts, the same shortcut-string grammar, conflict detection, and user
//! overrides. The differences are mechanical:
//!
//! - Persistence is the per-machine `ui_prefs.json` (`keybindings` map) instead
//!   of `localStorage` (mirrors every other Slint appearance pref).
//! - Key events come from winit (`on_winit_window_event`) instead of a DOM
//!   `keydown` listener. The `isInputTarget` guard becomes `UiFocusState`.
//! - The two modals + the capture widget are Slint (`KeybindingsState` /
//!   `KeyboardShortcutsState`); this module owns the model + dispatch.
//!
//! Grammar (identical canonical strings to the TS `eventToShortcut`):
//! `[Ctrl+][Alt+][Shift+]Key`. `Ctrl` covers Ctrl OR Meta/Super. `Shift` is
//! only emitted for letters, digits, and named keys (Arrow*, Space, …) — for a
//! symbol the Shift is already "consumed" by producing the symbol (e.g. `?`).

mod actions;
mod bindings;
mod capture;
mod dispatch;
mod grammar;
mod model;
mod mods;
mod wire;

pub use capture::handle_capture;
pub use dispatch::dispatch;
pub use grammar::token_from_key;
pub use mods::{mods, set_mods};
pub use wire::wire;
