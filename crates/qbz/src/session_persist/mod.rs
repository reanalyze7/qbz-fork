//! Session persistence (queue + playback state) — the Slint wiring for the
//! `persist_session` / `resume_playback_position` playback preferences.
//!
//! The on-disk store is the frontend-agnostic [`qbz_app::session_store::SessionStore`]
//! in `qbz-app` (a per-user `session.db`, already built + tested). This module is
//! the thin frontend glue: it owns a process-global store handle (bound at
//! session activation), captures the live queue + playback state into the
//! persisted snapshot at meaningful edges, and restores it at startup.
//!
//! Gating: `persist_session` (restore the queue/session) and
//! `resume_playback_position` (also restore the exact position) are per-user
//! playback preferences. Both are cached here so the hot save path never reopens
//! the prefs DB; [`set_gates`] refreshes the cache when the toggles change, and
//! [`init_for_user`] seeds them synchronously when the store opens (no race with
//! the async settings snapshot load).
//!
//! Phase A (this module) restores the queue + current track PAUSED — it touches
//! NO protected-audio code beyond threading an existing `start_position_secs`.
//! The saved position rides along via [`take_resume_for`] and is consumed on the
//! first play of the restored track, reusing the player's session-resume offset.

mod convert;
mod init;
mod restore;
mod save;
mod state;

pub use init::{init_for_user, save_on_exit};
pub use restore::restore;
pub use save::{capture_and_save, save_position};
pub use state::{
    bind_exit_ctx, pending_resume_position, set_gates, take_resume_for,
};
