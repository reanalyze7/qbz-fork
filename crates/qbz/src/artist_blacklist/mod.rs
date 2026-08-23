//! Per-user artist-blacklist lifecycle + access wrapper.
//!
//! A process-global singleton over the headless
//! `qbz_app::settings::artist_blacklist::BlacklistService` (ADR-006: all model
//! logic — schema, O(1) lookup set, enable flag, mutations — lives in
//! `qbz-app`; this module only owns the per-user store lifecycle and the thin
//! accessors the Slint surfaces call).
//!
//! Lifecycle mirrors `fav_cache` / `discover_prefs`: a process-global
//! `Mutex<Option<Service>>` bound per session via [`init_for_user`] /
//! [`teardown`], next to the other per-user stores. The service keeps its own
//! in-memory `HashSet` + enabled flag, so reads never round-trip SQLite; there
//! is no separate cache here and — matching `fav_cache` — no change-notify
//! mechanism: callers re-read after mutating (later UI tasks re-push Slint state
//! after a mutation).
//!
//! Fail-open everywhere: with no session bound (`None`), checks behave as "not
//! blacklisted" / "enabled", snapshots are empty, and mutations return the exact
//! Tauri error string so the UI shows the same message.

mod checks;
mod lifecycle;
mod mutations;
mod queries;
#[cfg(test)]
mod tests;

pub use checks::{
    card_blacklisted, is_album_blacklisted, is_blacklisted, is_blacklisted_id_str, is_enabled,
    is_track_blacklisted, stamp_row,
};
pub use lifecycle::{init_for_user, teardown};
pub use mutations::{
    add, add_album, clear_all, clear_all_albums, remove, remove_album, set_enabled,
};
pub use queries::{
    album_count, album_ids_snapshot, count, get_all, get_all_albums, ids_snapshot,
};
