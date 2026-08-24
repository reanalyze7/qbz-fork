//! Per-user pinned-items lifecycle + access wrapper.
//!
//! A process-global singleton over the headless
//! `qbz_app::settings::pinned_items::PinnedItemsService` (ADR-006: all model
//! logic — schema, O(1) key set, mutations — lives in `qbz-app`; this module
//! only owns the per-user store lifecycle and the thin accessors the Slint
//! surfaces call).
//!
//! Lifecycle mirrors `artist_blacklist` / `fav_cache` / `discover_prefs`: a
//! process-global `Mutex<Option<Service>>` bound per session via
//! [`init_for_user`] / [`teardown`], next to the other per-user stores. The
//! service keeps its own in-memory `(kind, id)` set, so reads never round-trip
//! SQLite; matching the family, there is no change-notify mechanism — mutation
//! sites re-run the consumer's reload / re-push path.
//!
//! Fail-open everywhere: with no session bound (`None`), checks behave as "not
//! pinned", the list/snapshot are empty, and mutations return the exact error
//! string the sibling stores use so the UI shows the same message.

mod lifecycle;
mod read;
#[cfg(test)]
mod tests;
mod write;

pub use lifecycle::{init_for_user, teardown};
pub use read::{is_pinned, list};
pub use write::{pin, unpin};

use std::sync::Mutex;

use qbz_app::settings::pinned_items::PinnedItemsService;

pub use qbz_app::settings::pinned_items::PinnedItem;

/// Per-user pinned-items service. `None` outside an active session (online or
/// offline); pure fail-open behavior in that window.
pub(super) static SERVICE: Mutex<Option<PinnedItemsService>> = Mutex::new(None);
