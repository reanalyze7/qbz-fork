//! Secure credential storage with fallback
//!
//! The encrypted AES-256-GCM file in the app config directory is the source
//! of truth for every credential. The OS keyring is used as an optional
//! best-effort cache:
//!
//! - Writes always go to the file first; the keyring is written opportunistically.
//! - Reads try the keyring first (cheaper when it works) and fall back to the file.
//! - Any keyring operation that fails or times out marks the keyring as broken
//!   for the rest of the process, so later reads/writes skip it entirely.
//!
//! This matters on Linux systems where GNOME Keyring / KWallet may be locked
//! with a password that no longer matches the current session (see issue #329).
//! Without the timeout + session memoization, every login triggers a blocking
//! "unlock keyring" dialog and the user has to dismiss 3-4 prompts per session.
//! With them, the dialog appears at most once per session and the app continues
//! through the encrypted file path regardless of what the user does with it.

mod crypto;
mod fallback_file;
mod keyring;
mod keys;
mod machine_salt;
mod oauth_token;
mod paths;
mod private_file;
mod qobuz_credentials;

#[cfg(test)]
mod tests;

pub use oauth_token::{
    clear_oauth_token, clear_oauth_token_at, load_oauth_token, load_oauth_token_at,
    load_oauth_token_from_file, oauth_token_file_present_at, save_oauth_token, save_oauth_token_at,
};
pub use qobuz_credentials::{
    clear_qobuz_credentials, has_saved_credentials, load_qobuz_credentials, save_qobuz_credentials,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QobuzCredentials {
    pub email: String,
    pub password: String,
}
