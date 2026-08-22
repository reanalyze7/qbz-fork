//! OAuth token persistence.
//!
//! OAuth login produces a `user_auth_token` instead of email+password.
//! We persist it encrypted the same way as regular credentials so the user
//! doesn't have to re-authenticate via browser on every app start.
//! The token is re-used at bootstrap via `POST /user/login` with the
//! `X-User-Auth-Token` header. If it has expired Qobuz returns a 4xx and
//! we clear the stored token so the user sees the login screen normally.
//!
//! ─── Shared OAuth-token file operations (root-parameterized) ──────────────
//!
//! The encrypted file is the authoritative store for both the desktop and the
//! daemon. This module's `write_oauth_token_file`/`read_oauth_token_file`/
//! `remove_oauth_token_file` do the file work under a caller-supplied root;
//! the keyring accelerator (desktop only) is layered on top by `desktop.rs`.
//! The `daemon.rs` `_at` fns skip the keyring entirely (01 §6.3: file-first is
//! authoritative; the keyring stays a desktop-only accelerator) and derive
//! their key with `PortalKey::Never` for the very same reason: an
//! init-started service reaches neither the Secret Service nor the XDG
//! portal, so anything session-scoped is unreadable there.

mod daemon;
mod desktop;

pub use daemon::{clear_oauth_token_at, load_oauth_token_at, save_oauth_token_at};
pub use desktop::{
    clear_oauth_token, load_oauth_token, load_oauth_token_from_file, save_oauth_token,
};

use std::fs;
use std::path::Path;

use crate::crypto::{decrypt_credentials_at, encrypt_credentials_at};
use crate::keys::PortalKey;
use crate::paths::oauth_token_path_at;
use crate::private_file::{tighten_private_file_mode, write_private_file};
use crate::QobuzCredentials;

pub(crate) const OAUTH_TOKEN_KEY: &str = "qobuz-oauth-token";

/// Encrypt `token` and write it to `<root>/.qbz-oauth-token` (0600). Returns the
/// encrypted blob so the caller can also mirror it into the keyring if desired.
pub(crate) fn write_oauth_token_file(
    root: &Path,
    portal: PortalKey,
    token: &str,
) -> Result<String, String> {
    let placeholder = QobuzCredentials {
        email: token.to_string(),
        password: String::new(),
    };
    let encrypted = encrypt_credentials_at(root, portal, &placeholder)?;
    write_private_file(&oauth_token_path_at(root), &encrypted)?;
    Ok(encrypted)
}

/// Read and decrypt the OAuth token from `<root>/.qbz-oauth-token`, or `None`
/// when the file is absent/empty/undecryptable.
///
/// On the daemon profile a failed decrypt is retried once with the session key
/// and, on success, rewritten portal-free — see the migration note inline.
pub(crate) fn read_oauth_token_file(
    root: &Path,
    portal: PortalKey,
) -> Result<Option<String>, String> {
    let path = oauth_token_path_at(root);
    if !path.exists() {
        return Ok(None);
    }

    tighten_private_file_mode(&path);
    let content =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read OAuth token file: {}", e))?;
    if content.trim().is_empty() {
        return Ok(None);
    }

    match decrypt_credentials_at(root, portal, &content) {
        Ok(placeholder) => {
            log::debug!("[Credentials] OAuth token loaded from encrypted file");
            Ok(Some(placeholder.email))
        }
        Err(e) => {
            // Daemon profile, one-shot migration: a token written while the
            // daemon still mixed the portal secret in decrypts with the session
            // key, but ONLY from a process that can reach the portal (a systemd
            // USER unit, say). Accept it once and rewrite it portal-free so
            // every later headless start can read it too. When no portal is
            // reachable both keys are identical, so this retry simply fails
            // again and we fall through to the warning.
            if portal == PortalKey::Never {
                if let Ok(placeholder) = decrypt_credentials_at(root, PortalKey::Session, &content) {
                    log::info!(
                        "[Credentials] Migrating the OAuth token to a session-independent key"
                    );
                    if let Err(e) =
                        write_oauth_token_file(root, PortalKey::Never, &placeholder.email)
                    {
                        log::warn!("[Credentials] Could not rewrite the migrated token: {}", e);
                    }
                    return Ok(Some(placeholder.email));
                }
            }
            log::warn!("[Credentials] Failed to decrypt OAuth token file: {}", e);
            Ok(None)
        }
    }
}

/// True when `<root>/.qbz-oauth-token` exists and is non-empty.
///
/// Lets a caller tell "no saved token" apart from "saved token this process
/// cannot decrypt" — `load_oauth_token_at` reports both as `None` so that a
/// broken file can never abort boot.
pub fn oauth_token_file_present_at(root: &Path) -> bool {
    let path = oauth_token_path_at(root);
    fs::read_to_string(&path)
        .map(|c| !c.trim().is_empty())
        .unwrap_or(false)
}

/// Remove `<root>/.qbz-oauth-token` if present.
pub(crate) fn remove_oauth_token_file(root: &Path) -> Result<(), String> {
    let path = oauth_token_path_at(root);
    if path.exists() {
        fs::remove_file(&path)
            .map_err(|e| format!("Failed to remove OAuth token file: {}", e))?;
    }
    Ok(())
}
