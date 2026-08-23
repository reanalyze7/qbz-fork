//! My QBZ — collection detail EDIT operations (Phase-2 Slice 7).
//!
//! Wires the hero overflow (⋯) menu + the Rename / Description / Delete-confirm
//! modals to the shared `qbz_mixtape::repo` setters, reached directly through
//! `crate::library_db::with_db` (ADR-005/006 — no Tauri command wrappers). Each
//! mutation mirrors its Tauri command (spec 40 §3.5/§3.6) and then RELOADS the
//! open detail view so the hero + state reflect the change (Tauri's
//! "-> reload"):
//!
//! - **Rename** (`v2_rename_mixtape_collection`): trim; empty -> no-op.
//! - **Description** (`v2_set_mixtape_description`): empty -> NULL (clear).
//! - **Play-mode toggle** (`v2_set_mixtape_play_mode`): in_order <-> album_shuffle.
//! - **Convert kind** (`v2_set_mixtape_kind`): mixtape <-> collection; the repo
//!   REJECTS any artist_collection conversion -> the "Cannot convert this kind"
//!   toast. Success -> "Converted".
//! - **Delete** (`v2_delete_mixtape_collection`, CASCADE): navigate BACK (which
//!   re-applies the previous grid entry, dropping the deleted row) on success;
//!   "Failed to delete" toast on error.
//!
//! All DB work runs synchronously inside `with_db` on a `spawn_blocking` worker
//! (no `&Connection` crosses an `.await`); the reload + toast hop back to the
//! event loop.

mod actions;
mod modal;
mod reload;

pub use actions::*;

// ──────────────────────────── DB write helpers ────────────────────────

/// Run a repo mutation that returns `rusqlite::Result<()>` against the per-user
/// library.db. Returns `Ok(())` on success, `Err(message)` on any failure
/// (DB unavailable or the repo error). Synchronous (`with_db`).
pub(crate) fn with_repo<F>(f: F) -> Result<(), String>
where
    F: FnOnce(&rusqlite::Connection) -> rusqlite::Result<()> + Send,
{
    match crate::library_db::with_db(|db| Ok(db.with_connection(f))) {
        Some(Ok(())) => Ok(()),
        Some(Err(e)) => Err(e.to_string()),
        None => Err("library database unavailable".to_string()),
    }
}
