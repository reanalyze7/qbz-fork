use super::UserDataPaths;
use std::path::PathBuf;
use std::sync::RwLock;

impl UserDataPaths {
    pub fn new() -> Self {
        Self {
            user_id: RwLock::new(None),
        }
    }

    /// Set the current user after login.
    pub fn set_user(&self, user_id: u64) {
        *self
            .user_id
            .write()
            .expect("UserDataPaths write lock poisoned") = Some(user_id);
        log::info!("UserDataPaths: active user set");
    }

    /// Clear the current user on logout.
    pub fn clear_user(&self) {
        *self
            .user_id
            .write()
            .expect("UserDataPaths write lock poisoned") = None;
        log::info!("UserDataPaths: active user cleared");
    }

    /// Get the current user ID, if set.
    pub fn current_user_id(&self) -> Option<u64> {
        *self
            .user_id
            .read()
            .expect("UserDataPaths read lock poisoned")
    }

    /// Get the user-scoped data directory: ~/.local/share/qbz/users/{uid}/
    pub fn user_data_dir(&self) -> Result<PathBuf, String> {
        let uid = self
            .user_id
            .read()
            .map_err(|e| format!("UserDataPaths read lock error: {}", e))?
            .ok_or("No active user - please log in")?;

        let base = dirs::data_dir()
            .ok_or("Could not determine data directory")?
            .join("qbz")
            .join("users")
            .join(uid.to_string());

        Ok(base)
    }

    /// Get the user-scoped cache directory: ~/.cache/qbz/users/{uid}/
    pub fn user_cache_dir(&self) -> Result<PathBuf, String> {
        let uid = self
            .user_id
            .read()
            .map_err(|e| format!("UserDataPaths read lock error: {}", e))?
            .ok_or("No active user - please log in")?;

        let base = dirs::cache_dir()
            .ok_or("Could not determine cache directory")?
            .join("qbz")
            .join("users")
            .join(uid.to_string());

        Ok(base)
    }

    /// Data directory for an ARBITRARY user id (no active-user requirement):
    /// ~/.local/share/qbz/users/{uid}/ — the same layout `user_data_dir`
    /// resolves for the active user. Used by the guest-profile adoption
    /// (#553), which must compare two users' paths before either is active.
    pub fn data_dir_for(user_id: u64) -> Result<PathBuf, String> {
        Ok(Self::global_data_dir()?
            .join("users")
            .join(user_id.to_string()))
    }

    /// Cache twin of [`Self::data_dir_for`]: ~/.cache/qbz/users/{uid}/.
    pub fn cache_dir_for(user_id: u64) -> Result<PathBuf, String> {
        Ok(Self::global_cache_dir()?
            .join("users")
            .join(user_id.to_string()))
    }

    /// Get the global (non-user-scoped) data directory: ~/.local/share/qbz/
    pub fn global_data_dir() -> Result<PathBuf, String> {
        dirs::data_dir()
            .ok_or_else(|| "Could not determine data directory".to_string())
            .map(|d| d.join("qbz"))
    }

    /// Get the global (non-user-scoped) cache directory: ~/.cache/qbz/
    pub fn global_cache_dir() -> Result<PathBuf, String> {
        dirs::cache_dir()
            .ok_or_else(|| "Could not determine cache directory".to_string())
            .map(|d| d.join("qbz"))
    }
}
