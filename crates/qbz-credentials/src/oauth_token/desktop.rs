//! Desktop-profile OAuth token API: fixed `~/.config/qbz` root, file
//! authoritative with the keyring layered on top as a best-effort cache.

use crate::crypto::decrypt_credentials;
use crate::keyring::{keyring_delete, keyring_get, keyring_set};
use crate::keys::PortalKey;
use crate::paths::config_qbz_root;

use super::{
    read_oauth_token_file, remove_oauth_token_file, write_oauth_token_file, OAUTH_TOKEN_KEY,
};

/// Persist the OAuth `user_auth_token`.
///
/// File is authoritative: the encrypted token is written to the config
/// directory unconditionally. The keyring is a best-effort write-through
/// cache — if it fails (or times out behind an unlock dialog), the login
/// flow still completes because the file is already on disk. Inverts the
/// previous keyring-first ordering, which forced a prompt on every login
/// for users with a broken Secret Service collection (issue #329).
pub fn save_oauth_token(token: &str) -> Result<(), String> {
    let root = config_qbz_root().ok_or("Could not determine config directory")?;
    let encrypted = write_oauth_token_file(&root, PortalKey::Session, token)?;
    log::info!("[Credentials] OAuth token saved to encrypted file");

    if keyring_set(OAUTH_TOKEN_KEY, &encrypted) {
        log::debug!("[Credentials] OAuth token also saved to keyring");
    }

    Ok(())
}

/// Load a previously saved OAuth `user_auth_token`, or `None` if absent.
/// Prefers the keyring when it responds quickly, otherwise reads the file.
pub fn load_oauth_token() -> Result<Option<String>, String> {
    if let Some(encrypted) = keyring_get(OAUTH_TOKEN_KEY) {
        if !encrypted.is_empty() {
            if let Ok(placeholder) = decrypt_credentials(&encrypted) {
                log::debug!("[Credentials] OAuth token loaded from keyring");
                return Ok(Some(placeholder.email));
            }
            // Legacy format: pre-encryption builds stored the raw token in
            // the keyring. Accept it for this one read; the next successful
            // `save_oauth_token` call will rewrite it encrypted to both the
            // keyring and the file.
            log::debug!("[Credentials] Keyring held legacy plaintext token; will re-encrypt on next save");
            return Ok(Some(encrypted));
        }
    }

    load_oauth_token_from_file()
}

/// Load the OAuth token from the encrypted file ONLY, skipping the keyring.
///
/// The file is the authoritative store (`save_oauth_token` writes it first);
/// the keyring is a best-effort cache that can go stale — e.g. an entry
/// encrypted under an older key shadows a perfectly fresh file because
/// `load_oauth_token` prefers the keyring. Callers that just watched a
/// keyring-sourced token fail authentication (and diagnostic tooling) can
/// use this to read the authoritative copy directly.
pub fn load_oauth_token_from_file() -> Result<Option<String>, String> {
    match config_qbz_root() {
        Some(root) => read_oauth_token_file(&root, PortalKey::Session),
        None => Ok(None),
    }
}

/// Delete the stored OAuth token (logout or token expiry).
pub fn clear_oauth_token() -> Result<(), String> {
    keyring_delete(OAUTH_TOKEN_KEY);

    if let Some(root) = config_qbz_root() {
        remove_oauth_token_file(&root)?;
        log::info!("[Credentials] OAuth token file cleared");
    }
    Ok(())
}
