use rand::RngCore;

use crate::error::SecretError;

use super::{KEYRING_ENTRY_NAME, MASTER_KEY_LEN};

/// Read (or create and store) the master key from the OS keyring.
pub(super) fn try_open_keyring(service_name: &str) -> Result<[u8; MASTER_KEY_LEN], SecretError> {
    use base64::engine::general_purpose::STANDARD as B64;
    use base64::Engine;
    use keyring::Entry;

    let entry = Entry::new(service_name, KEYRING_ENTRY_NAME)
        .map_err(|e| SecretError::Keyring(format!("Entry::new: {}", e)))?;

    match entry.get_password() {
        Ok(existing_b64) => {
            let bytes = B64
                .decode(existing_b64.trim())
                .map_err(|e| SecretError::Keyring(format!("base64 decode: {}", e)))?;
            if bytes.len() != MASTER_KEY_LEN {
                return Err(SecretError::Keyring(format!(
                    "keyring entry has wrong length ({} bytes, expected {})",
                    bytes.len(),
                    MASTER_KEY_LEN
                )));
            }
            let mut out = [0u8; MASTER_KEY_LEN];
            out.copy_from_slice(&bytes);
            Ok(out)
        }
        Err(keyring::Error::NoEntry) => {
            let mut key = [0u8; MASTER_KEY_LEN];
            rand::rng().fill_bytes(&mut key);
            let encoded = B64.encode(key);
            entry
                .set_password(&encoded)
                .map_err(|e| SecretError::Keyring(format!("set_password: {}", e)))?;
            log::info!(
                "[qbz-secrets] Generated fresh 256-bit master key and stored in OS keyring"
            );
            Ok(key)
        }
        Err(e) => Err(SecretError::Keyring(format!("get_password: {}", e))),
    }
}
