//! Mutations (Err with the "no active session" string when unbound).

use qbz_app::settings::pinned_items::PinnedItemsService;

use super::{PinnedItem, SERVICE};

/// The exact error string the sibling per-user stores return for a mutation
/// attempted with no active session. Kept verbatim so the UI surfaces the
/// same message.
const NO_SESSION_ERR: &str = "No active session - please log in";

/// Run a mutation against the bound service, returning the "no active
/// session" error string when there is none / the lock is poisoned.
fn mutate(f: impl FnOnce(&PinnedItemsService) -> Result<(), String>) -> Result<(), String> {
    match SERVICE.lock() {
        Ok(guard) => match guard.as_ref() {
            Some(service) => f(service),
            None => Err(NO_SESSION_ERR.into()),
        },
        Err(_) => Err(NO_SESSION_ERR.into()),
    }
}

/// Pin an item (upsert; `pinned_at` is stamped by the service).
pub fn pin(item: &PinnedItem) -> Result<(), String> {
    mutate(|s| s.pin(item))
}

/// Unpin an item. Absent rows are Ok, not an error.
pub fn unpin(kind: &str, id: &str) -> Result<(), String> {
    mutate(|s| s.unpin(kind, id))
}
