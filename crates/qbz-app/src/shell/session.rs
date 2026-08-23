use std::path::Path;

use qbz_core::FrontendAdapter;

use super::{ActiveSession, AppRuntime};
use crate::session_store::SessionStore;
use crate::user_data::UserDataPaths;

impl<A: FrontendAdapter + Send + Sync + 'static> AppRuntime<A> {
    // ==================== Session activation (Task 2) ====================

    /// Activate the per-user session against explicit directories.
    ///
    /// This is the testable core of session activation. It creates the
    /// directories, opens the session store, and marks the runtime state
    /// machine as session-activated. It performs no global-path writes (no
    /// `last_user_id` marker) and does not touch [`UserDataPaths`] state, so
    /// tests and shells managing their own paths can call it directly.
    pub async fn activate_at(
        &self,
        user_id: u64,
        data_dir: &Path,
        cache_dir: &Path,
    ) -> Result<(), String> {
        std::fs::create_dir_all(data_dir)
            .map_err(|e| format!("Failed to create user data dir: {}", e))?;
        std::fs::create_dir_all(cache_dir)
            .map_err(|e| format!("Failed to create user cache dir: {}", e))?;

        let session_store = SessionStore::new_at(data_dir)?;

        self.runtime.set_session_activated(true, user_id).await;

        let mut guard = self
            .session
            .lock()
            .map_err(|e| format!("session lock poisoned: {}", e))?;
        *guard = Some(ActiveSession {
            user_id,
            session_store,
        });

        log::info!("[AppRuntime] Session activated for user");
        Ok(())
    }

    /// Activate the per-user session for `user_id`.
    ///
    /// Resolves the real per-user directories through [`UserDataPaths`],
    /// activates against them, and persists the last-user marker so the
    /// session can be restored on the next launch.
    pub async fn activate(&self, user_id: u64) -> Result<(), String> {
        Self::adopt_guest_profile(user_id);
        self.user_paths.set_user(user_id);
        let data_dir = self.user_paths.user_data_dir()?;
        let cache_dir = self.user_paths.user_cache_dir()?;
        self.activate_at(user_id, &data_dir, &cache_dir).await?;
        UserDataPaths::save_last_user_id(user_id)?;
        Ok(())
    }

    /// Activate an offline-only session using the last known user.
    ///
    /// Falls back to user id `0` (an empty profile) when no previous session
    /// was recorded. Does not re-persist the last-user marker.
    pub async fn activate_offline(&self) -> Result<(), String> {
        let user_id = UserDataPaths::load_last_user_id().unwrap_or(0);
        self.user_paths.set_user(user_id);
        let data_dir = self.user_paths.user_data_dir()?;
        let cache_dir = self.user_paths.user_cache_dir()?;
        self.activate_at(user_id, &data_dir, &cache_dir).await
    }

    /// Deactivate the current session.
    ///
    /// Drops the open per-user stores (closing their database connections),
    /// clears the active user, and resets the runtime state machine. The
    /// `last_user_id` marker is intentionally kept on disk so a later
    /// offline session can still find the user's data.
    pub async fn deactivate(&self) -> Result<(), String> {
        {
            let mut guard = self
                .session
                .lock()
                .map_err(|e| format!("session lock poisoned: {}", e))?;
            *guard = None;
        }
        self.user_paths.clear_user();
        self.runtime.set_session_activated(false, 0).await;
        log::info!("[AppRuntime] Session deactivated");
        Ok(())
    }

    /// Whether a per-user session is currently active.
    pub fn is_session_active(&self) -> bool {
        self.session
            .lock()
            .map(|guard| guard.is_some())
            .unwrap_or(false)
    }

    /// The active user id, if a session is active.
    pub fn active_user_id(&self) -> Option<u64> {
        self.session
            .lock()
            .ok()
            .and_then(|guard| guard.as_ref().map(|s| s.user_id))
    }

    /// Run a closure with the active session store.
    ///
    /// Returns `None` when no session is active. This hands the shell the
    /// real [`SessionStore`] API without duplicating its methods on the
    /// facade.
    pub fn with_session_store<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&SessionStore) -> R,
    {
        let guard = self.session.lock().ok()?;
        guard.as_ref().map(|s| f(&s.session_store))
    }
}
