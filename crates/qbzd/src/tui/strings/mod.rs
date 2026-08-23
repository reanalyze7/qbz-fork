// crates/qbzd/src/tui/strings/ — EVERY user-facing string in the setup TUI.
//
// English-only v1 (03-setup-tui.md §1.2). Centralized here so a later gettext
// pass (qbz-i18n is Slint-free) is a P2 batch job, not a rewrite. Nothing under
// tui/ prints a bare literal — it comes from here. One module per screen/
// section, mirroring the TUI's own screen names.
mod account;
mod audio;
mod bundle;
mod dirty_footer;
mod entry;
mod network;
mod playback;
mod save_result;
mod scrobbler;
mod shell;
mod wizard;

pub use account::*;
pub use audio::*;
pub use bundle::*;
pub use dirty_footer::*;
pub use entry::*;
pub use network::*;
pub use playback::*;
pub use save_result::*;
pub use scrobbler::*;
pub use shell::*;
pub use wizard::*;
