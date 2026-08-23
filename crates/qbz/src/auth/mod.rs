//! System-browser OAuth login for the Slint MVP.
//!
//! Mirrors the QBZ Tauri `v2_start_system_browser_oauth` flow without any
//! Tauri or WebView dependency: it opens the user's default browser to the
//! Qobuz OAuth page with a localhost redirect, captures the authorization
//! code on a one-shot local HTTP listener, exchanges it through the core
//! Qobuz client, and activates the per-user session via `AppRuntime`.

mod init;
mod login;
mod logout;
mod oauth_listener;
mod per_user;
mod restore;
mod types;

pub use login::login_via_system_browser;
pub use logout::logout;
pub use restore::restore_saved_session;
pub use types::{LoginPhase, SessionInfo};
