use std::path::Path;
use std::sync::atomic::Ordering;

use super::OfflineModeEngine;
use crate::offline_mode::store::OfflineModeStore;

impl OfflineModeEngine {
    /// Open the per-user settings store and load the persisted induced flag.
    /// Call on session activation (online or offline).
    pub fn init_for_user(&self, base_dir: &Path) -> Result<(), String> {
        let store = OfflineModeStore::new_at(base_dir)?;
        let induced = store.get_settings()?.manual_offline_mode;
        {
            let mut guard = self
                .store
                .lock()
                .map_err(|e| format!("offline store lock poisoned: {}", e))?;
            *guard = Some(store);
        }
        self.induced.store(induced, Ordering::Relaxed);
        self.recompute();
        Ok(())
    }

    /// Drop the per-user store AND end the session-scoped offline state
    /// (logout). Ending the session ends the session: `offline_session` is
    /// reset (it must not survive into the next login attempt — a surviving
    /// flag kept the Qobuz gate closed and refused the login itself), and the
    /// cached `induced` flag is reset too (no user ⇒ no induced opt-in
    /// active; the user's persisted preference reloads from disk on the next
    /// `init_for_user`). The final `recompute()` reopens the Qobuz gate when
    /// connectivity allows.
    pub fn teardown(&self) {
        if let Ok(mut guard) = self.store.lock() {
            *guard = None;
        }
        self.offline_session.store(false, Ordering::Relaxed);
        self.induced.store(false, Ordering::Relaxed);
        self.recompute();
    }
}
