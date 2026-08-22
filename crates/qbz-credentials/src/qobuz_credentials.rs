//! The public Qobuz email+password API: thin orchestration over the
//! encrypted fallback file (authoritative) and the keyring (best-effort
//! write-through cache).

use crate::fallback_file::{clear_fallback, has_fallback_credentials, load_from_fallback, save_to_fallback};
use crate::keyring::{keyring_delete, keyring_get, keyring_set};
use crate::QobuzCredentials;

const QOBUZ_CREDENTIALS_KEY: &str = "qobuz-credentials";

/// Save Qobuz email+password credentials.
///
/// File is authoritative: we write it first and fail the operation if that
/// fails. The keyring is a best-effort write-through cache.
pub fn save_qobuz_credentials(email: &str, password: &str) -> Result<(), String> {
    log::info!("[Credentials] Saving Qobuz credentials");

    let credentials = QobuzCredentials {
        email: email.to_string(),
        password: password.to_string(),
    };

    save_to_fallback(&credentials)?;

    let json = serde_json::to_string(&credentials).unwrap_or_default();
    if !json.is_empty() && keyring_set(QOBUZ_CREDENTIALS_KEY, &json) {
        log::debug!("[Credentials] Qobuz credentials also saved to keyring");
    }

    Ok(())
}

/// Load Qobuz email+password credentials. Prefers the keyring when it
/// responds quickly, otherwise reads the encrypted fallback file.
pub fn load_qobuz_credentials() -> Result<Option<QobuzCredentials>, String> {
    log::debug!("[Credentials] Loading Qobuz credentials");

    if let Some(json) = keyring_get(QOBUZ_CREDENTIALS_KEY) {
        match serde_json::from_str::<QobuzCredentials>(&json) {
            Ok(credentials) => {
                log::debug!("[Credentials] Loaded Qobuz credentials from keyring");
                return Ok(Some(credentials));
            }
            Err(e) => {
                log::warn!(
                    "[Credentials] Keyring entry could not be parsed ({}), falling back to file",
                    e
                );
            }
        }
    }

    load_from_fallback()
}

/// Report whether any saved Qobuz credentials exist (keyring or file).
pub fn has_saved_credentials() -> bool {
    if keyring_get(QOBUZ_CREDENTIALS_KEY).is_some() {
        return true;
    }
    has_fallback_credentials()
}

/// Clear saved Qobuz credentials from both the keyring and the fallback file.
pub fn clear_qobuz_credentials() -> Result<(), String> {
    keyring_delete(QOBUZ_CREDENTIALS_KEY);
    clear_fallback()?;
    Ok(())
}
