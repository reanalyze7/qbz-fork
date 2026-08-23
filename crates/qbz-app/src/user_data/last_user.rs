use super::UserDataPaths;
use std::path::PathBuf;

impl UserDataPaths {
    /// Save the last active user_id to a flat-path file so the session can be
    /// restored on next app launch when remember-me is active.
    pub fn save_last_user_id(user_id: u64) -> Result<(), String> {
        let path = Self::last_user_id_path()?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)
                .map_err(|e| format!("Failed to create global data directory: {}", e))?;
        }
        std::fs::write(&path, user_id.to_string())
            .map_err(|e| format!("Failed to save last user id: {}", e))?;
        log::info!("Saved last_user_id marker");
        Ok(())
    }

    /// Read the last active user_id. Returns None if the file is missing or
    /// invalid.
    pub fn load_last_user_id() -> Option<u64> {
        let path = Self::last_user_id_path().ok()?;
        let contents = std::fs::read_to_string(&path).ok()?;
        contents.trim().parse::<u64>().ok()
    }

    /// Clear the last user_id file, called on explicit logout.
    pub fn clear_last_user_id() {
        if let Ok(path) = Self::last_user_id_path() {
            let _ = std::fs::remove_file(&path);
            log::info!("Cleared last_user_id file");
        }
    }

    fn last_user_id_path() -> Result<PathBuf, String> {
        let dir = Self::global_data_dir()?;
        Ok(dir.join("last_user_id"))
    }
}
