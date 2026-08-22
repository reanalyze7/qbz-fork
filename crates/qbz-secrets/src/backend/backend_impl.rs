use std::path::Path;

use crate::error::SecretError;

use super::kdf::derive_fallback_key;
use super::keyring::try_open_keyring;
use super::kind::BackendKind;
use super::{BACKEND_MARKER_KDF, BACKEND_MARKER_KEYRING, MASTER_KEY_LEN};

use crate::cipher::{unwrap_with_key, wrap_with_key};

pub struct Backend {
    kind: BackendKind,
    master_key: [u8; MASTER_KEY_LEN],
}

impl Backend {
    pub fn new(service_name: &str, storage_dir: &Path) -> Result<Self, SecretError> {
        // Try keyring first.
        match try_open_keyring(service_name) {
            Ok(master_key) => {
                log::info!("[qbz-secrets] Using OS keyring backend");
                return Ok(Self {
                    kind: BackendKind::Keyring,
                    master_key,
                });
            }
            Err(e) => {
                log::warn!(
                    "[qbz-secrets] OS keyring unavailable ({}) — falling back to KDF-derived key",
                    e
                );
            }
        }

        // Fallback: derive from device identifiers.
        let master_key = derive_fallback_key(service_name, storage_dir)?;
        Ok(Self {
            kind: BackendKind::KdfFallback,
            master_key,
        })
    }

    pub fn kind(&self) -> BackendKind {
        self.kind
    }

    pub fn wrap(&self, plaintext: &[u8]) -> Result<Vec<u8>, SecretError> {
        let marker = match self.kind {
            BackendKind::Keyring => BACKEND_MARKER_KEYRING,
            BackendKind::KdfFallback => BACKEND_MARKER_KDF,
        };
        wrap_with_key(&self.master_key, marker, plaintext)
    }

    pub fn unwrap(&self, wrapped: &[u8]) -> Result<Vec<u8>, SecretError> {
        let expected = match self.kind {
            BackendKind::Keyring => BACKEND_MARKER_KEYRING,
            BackendKind::KdfFallback => BACKEND_MARKER_KDF,
        };
        unwrap_with_key(&self.master_key, expected, wrapped)
    }
}
