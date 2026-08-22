//! Daemon-profile OAuth token API: root-parameterized, file-only (no
//! keyring), always derived with `PortalKey::Never` — see `super`'s doc
//! comment for why the daemon can never touch the keyring or the portal.

use std::path::Path;

use super::{read_oauth_token_file, remove_oauth_token_file, write_oauth_token_file};
use crate::keys::PortalKey;

/// Persist the OAuth `user_auth_token` under `root` (daemon path, file-only).
///
/// File is authoritative; no keyring write-through (01 §6.3 — the keyring
/// accelerator stays desktop-only, so a daemon runs with no Secret Service).
pub fn save_oauth_token_at(root: &Path, token: &str) -> Result<(), String> {
    write_oauth_token_file(root, PortalKey::Never, token)?;
    log::info!("[Credentials] OAuth token saved to encrypted file (daemon root)");
    Ok(())
}

/// Load a previously saved OAuth `user_auth_token` from under `root`, or `None`.
/// File-only (no keyring), the authoritative daemon path.
pub fn load_oauth_token_at(root: &Path) -> Result<Option<String>, String> {
    read_oauth_token_file(root, PortalKey::Never)
}

/// Delete the stored OAuth token under `root` (daemon path, file-only).
pub fn clear_oauth_token_at(root: &Path) -> Result<(), String> {
    remove_oauth_token_file(root)?;
    log::info!("[Credentials] OAuth token file cleared (daemon root)");
    Ok(())
}
