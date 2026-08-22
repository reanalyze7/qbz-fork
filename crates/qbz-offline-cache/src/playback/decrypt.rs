//! Unwrap the content key via the secret vault and decrypt a loaded
//! bundle into playable FLAC bytes.

use std::path::Path;

use crate::cmaf_store::LoadedBundle;
use crate::secret_vault;

pub(super) fn unwrap_and_decrypt(
    track_id: u64,
    loaded: &LoadedBundle,
    content_key_wrapped: &[u8],
    offline_root_path: &Path,
) -> Option<Vec<u8>> {
    let vault = match secret_vault::get_or_init(offline_root_path) {
        Ok(v) => v,
        Err(e) => {
            log::warn!(
                "[OfflineCache/Play] Track {} SecretBox init failed: {}",
                track_id,
                e
            );
            return None;
        }
    };
    let unwrapped = match vault.unwrap(content_key_wrapped) {
        Ok(k) => k,
        Err(e) => {
            log::warn!(
                "[OfflineCache/Play] Track {} content_key unwrap failed: {}",
                track_id,
                e
            );
            return None;
        }
    };
    if unwrapped.len() != 16 {
        log::warn!(
            "[OfflineCache/Play] Track {} unwrapped content_key wrong size ({} bytes)",
            track_id,
            unwrapped.len()
        );
        return None;
    }
    let mut content_key = [0u8; 16];
    content_key.copy_from_slice(&unwrapped);

    match loaded.decrypt_to_flac(&content_key) {
        Ok(flac_bytes) => {
            log::info!(
                "[OfflineCache/Play] Track {} unwrapped + decrypted ({:.2} MB FLAC)",
                track_id,
                flac_bytes.len() as f64 / (1024.0 * 1024.0)
            );
            Some(flac_bytes)
        }
        Err(e) => {
            log::warn!(
                "[OfflineCache/Play] Track {} CMAF decrypt failed: {}",
                track_id,
                e
            );
            None
        }
    }
}
