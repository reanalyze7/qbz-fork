//! Writing to / clearing / probing the fallback file (not reading it back —
//! see `load.rs` for the read-with-migration path).

use std::fs;

use crate::crypto::encrypt_credentials;
use crate::paths::{get_fallback_path, get_legacy_fallback_path};
use crate::private_file::write_private_file;
use crate::QobuzCredentials;

/// Save credentials to fallback file (AES-256-GCM encrypted)
pub(crate) fn save_to_fallback(credentials: &QobuzCredentials) -> Result<(), String> {
    let path = get_fallback_path().ok_or("Could not determine config directory")?;

    // Ensure directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    let encrypted = encrypt_credentials(credentials)?;

    write_private_file(&path, encrypted)?;

    log::info!("Credentials saved to encrypted fallback file");
    Ok(())
}

/// Clear fallback credentials file
pub(crate) fn clear_fallback() -> Result<(), String> {
    if let Some(path) = get_fallback_path() {
        if path.exists() {
            fs::remove_file(&path)
                .map_err(|e| format!("Failed to remove credentials file: {}", e))?;
            log::info!("Fallback credentials file removed");
        }
    }
    // Also clear legacy file if exists
    if let Some(legacy_path) = get_legacy_fallback_path() {
        if legacy_path.exists() {
            let _ = fs::remove_file(&legacy_path);
        }
    }
    Ok(())
}

/// Check if fallback file exists
pub(crate) fn has_fallback_credentials() -> bool {
    get_fallback_path().map(|p| p.exists()).unwrap_or(false)
}
