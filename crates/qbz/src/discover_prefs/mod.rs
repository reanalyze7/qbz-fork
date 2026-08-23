//! Discover section-configurator controller (Slice 5).
//!
//! Thin frontend binding over the headless `qbz_app::settings::discover_prefs`
//! store (ADR-006: all model logic — defaults, reconcile, toggle, move, reset —
//! lives in `qbz-app`; this module only owns the per-user store lifecycle, the
//! in-memory authoritative copy, and the push helpers that map prefs into the
//! `DiscoverState` Slint global).
//!
//! Lifecycle mirrors `fav_cache`: a process-global `Mutex<Option<Store>>`
//! (persistence) + `Mutex<Option<Prefs>>` (in-memory authoritative, so a UI
//! toggle never round-trips SQLite on the event loop), bound per session via
//! [`init_for_user`] / [`teardown`] next to the other per-user stores.
//!
//! The render driver: Rust recomputes `prefs.enabled_ordered(tab)` on every
//! mutation and on tab switch, then pushes the ordered descriptor lists. For You
//! descriptors are built here (the data lives in `ForYouState`, dispatched by
//! id); Home / Editor's Picks descriptors are built in `crate::home` (it owns
//! the cached `SectionData` the album-carousel arms embed). The configurator
//! modal reads `config-rows` (the FULL ordered list, enabled + disabled).

mod descriptors;
mod labels;
mod mutations;
mod reco_ttl;

pub use descriptors::{push_config_rows, push_descriptors, seed};
pub use labels::{label_for, render_kind};
pub use mutations::{on_close_configurator, on_move, on_open_configurator, on_reset, on_toggle};
pub use reco_ttl::{reco_cache_ttl_secs, set_reco_cache_ttl_index, set_show_recommendations};

use std::path::Path;
use std::sync::Mutex;

use qbz_app::settings::discover_prefs::{default_prefs, DiscoverPrefs, DiscoverPrefsStore};

/// Per-user persistent store. `None` outside an active session.
pub(super) static STORE: Mutex<Option<DiscoverPrefsStore>> = Mutex::new(None);
/// In-memory authoritative prefs — the source of truth for the UI thread.
pub(super) static PREFS: Mutex<Option<DiscoverPrefs>> = Mutex::new(None);

// ---------------------------------------------------------------------------
// Lifecycle (mirrors fav_cache::{init_for_user, teardown})
// ---------------------------------------------------------------------------

/// Bind the per-user store and load the persisted prefs into memory. Called on
/// every session activation (login / restore / offline entry), next to
/// `fav_cache::init_for_user`. Best-effort: a store-open failure logs and falls
/// back to in-memory defaults (the configurator still works, just non-persistent).
pub fn init_for_user(base_dir: &Path) {
    match DiscoverPrefsStore::new_at(base_dir) {
        Ok(store) => {
            *PREFS.lock().unwrap() = Some(store.load());
            *STORE.lock().unwrap() = Some(store);
        }
        Err(e) => {
            log::error!("[qbz-slint] discover prefs store open failed: {e}");
            *PREFS.lock().unwrap() = Some(default_prefs());
        }
    }
}

/// Drop the per-user store and in-memory prefs on logout.
pub fn teardown() {
    *STORE.lock().unwrap() = None;
    *PREFS.lock().unwrap() = None;
}

/// A clone of the current in-memory prefs (defaults if no session yet).
pub(super) fn current() -> DiscoverPrefs {
    PREFS.lock().unwrap().clone().unwrap_or_else(default_prefs)
}

pub(super) fn persist() {
    if let (Some(p), Some(s)) = (
        PREFS.lock().unwrap().as_ref(),
        STORE.lock().unwrap().as_ref(),
    ) {
        if let Err(e) = s.save(p) {
            log::error!("[qbz-slint] discover prefs save failed: {e}");
        }
    }
}

/// Read-through used by `crate::home::select_tab` so a tab switch recomputes the
/// active tab's descriptors from the same prefs the controller owns.
pub fn prefs_snapshot() -> DiscoverPrefs {
    current()
}
