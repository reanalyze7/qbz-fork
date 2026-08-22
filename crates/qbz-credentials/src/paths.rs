//! File-name constants and path helpers for the credential store.

use std::path::{Path, PathBuf};

pub(crate) const FALLBACK_FILE_NAME: &str = ".qbz-auth";
pub(crate) const LEGACY_FALLBACK_FILE_NAME: &str = ".qbz-auth.legacy";
pub(crate) const OAUTH_TOKEN_FILE_NAME: &str = ".qbz-oauth-token";
pub(crate) const INSTALLATION_SALT_FILE_NAME: &str = ".qbz-cred-salt";
pub(crate) const MACHINE_ID_FALLBACK_FILE_NAME: &str = ".qbz-machine-id";

/// Get the fallback credentials file path
pub(crate) fn get_fallback_path() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("qbz").join(FALLBACK_FILE_NAME))
}

/// Get the legacy fallback file path (for migration)
pub(crate) fn get_legacy_fallback_path() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("qbz").join(LEGACY_FALLBACK_FILE_NAME))
}

// ─── Path-parameterized roots (daemon support, 01-architecture.md §6.3) ────────
//
// The desktop hardcodes `~/.config/qbz` for its secret files. The daemon owns a
// SEPARATE profile (`~/.config/qbzd`, §4) so a desktop logout can never silently
// de-auth the daemon. These helpers resolve the salt / machine-id-fallback /
// OAuth-token files directly under a caller-supplied root. Crypto, format and
// the KDF are unchanged — only the base directory is parameterized. The desktop
// fns below are thin wrappers that pass `~/.config/qbz` as the root, so their
// behaviour (including the keyring accelerator) is identical to before.

/// The desktop credential root: `~/.config/qbz`.
pub(crate) fn config_qbz_root() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("qbz"))
}

pub(crate) fn installation_salt_path_at(root: &Path) -> PathBuf {
    root.join(INSTALLATION_SALT_FILE_NAME)
}

pub(crate) fn machine_id_fallback_path_at(root: &Path) -> PathBuf {
    root.join(MACHINE_ID_FALLBACK_FILE_NAME)
}

pub(crate) fn oauth_token_path_at(root: &Path) -> PathBuf {
    root.join(OAUTH_TOKEN_FILE_NAME)
}
