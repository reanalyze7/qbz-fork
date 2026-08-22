//! Reading the fallback file, transparently migrating a legacy-format file
//! (or a stray `.qbz-auth.legacy`) to the current encrypted format as a
//! side effect of the read.

use std::fs;

use super::legacy::load_legacy_credentials;
use super::store::save_to_fallback;
use crate::crypto::decrypt_credentials;
use crate::paths::{get_fallback_path, get_legacy_fallback_path};
use crate::private_file::tighten_private_file_mode;
use crate::QobuzCredentials;

/// Load credentials from fallback file
pub(crate) fn load_from_fallback() -> Result<Option<QobuzCredentials>, String> {
    let path = match get_fallback_path() {
        Some(p) => p,
        None => return Ok(None),
    };

    if !path.exists() {
        // Check for legacy file and migrate if found
        if let Some(legacy_path) = get_legacy_fallback_path() {
            if legacy_path.exists() {
                log::info!("Found legacy credentials file, attempting migration...");
                if let Ok(Some(creds)) = load_legacy_credentials(&legacy_path) {
                    // Save in new format
                    if save_to_fallback(&creds).is_ok() {
                        // Remove legacy file
                        let _ = fs::remove_file(&legacy_path);
                        log::info!("Successfully migrated credentials to new encrypted format");
                        return Ok(Some(creds));
                    }
                }
            }
        }

        // Also check if the current file is in legacy format (migration from old .qbz-auth)
        let current_path = get_fallback_path();
        if let Some(ref p) = current_path {
            if p.exists() {
                // Try reading as JSON first (new format)
                if let Ok(content) = fs::read_to_string(p) {
                    if content.trim().starts_with('{') && content.contains("\"version\"") {
                        // It's the new format, will be handled below
                    } else {
                        // Might be legacy format
                        log::info!("Attempting to read legacy format from current file...");
                        if let Ok(Some(creds)) = load_legacy_credentials(p) {
                            // Save in new format
                            if save_to_fallback(&creds).is_ok() {
                                log::info!(
                                    "Successfully migrated credentials to new encrypted format"
                                );
                                return Ok(Some(creds));
                            }
                        }
                    }
                }
            }
        }

        return Ok(None);
    }

    tighten_private_file_mode(&path);
    let content =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read credentials file: {}", e))?;

    // Check if it's the new format or legacy
    if content.trim().starts_with('{') && content.contains("\"version\"") {
        // New encrypted format
        match decrypt_credentials(&content) {
            Ok(creds) => {
                log::info!("Credentials loaded from encrypted fallback file");
                Ok(Some(creds))
            }
            Err(e) => {
                log::warn!("Failed to decrypt credentials: {}", e);
                // Try legacy format as fallback
                if let Ok(Some(creds)) = load_legacy_credentials(&path) {
                    log::info!("Loaded from legacy format, will re-encrypt on next save");
                    return Ok(Some(creds));
                }
                Err(e)
            }
        }
    } else {
        // Legacy format - try to load and migrate
        log::info!("Found legacy format, migrating...");
        if let Ok(Some(creds)) = load_legacy_credentials(&path) {
            // Save in new format
            if save_to_fallback(&creds).is_ok() {
                log::info!("Successfully migrated credentials to new encrypted format");
            }
            return Ok(Some(creds));
        }
        Ok(None)
    }
}
