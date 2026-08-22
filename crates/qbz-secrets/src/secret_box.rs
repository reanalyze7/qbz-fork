//! [`SecretBox`]: the handle callers use to wrap/unwrap secrets.

use std::path::Path;

use crate::{Backend, BackendKind, SecretError};

/// Handle to the secret storage. Cheap to clone (reference-counted
/// internally via `Arc` inside the backend).
#[derive(Clone)]
pub struct SecretBox {
    backend: std::sync::Arc<Backend>,
}

impl SecretBox {
    /// Open the secret store. Tries the OS keyring first; if that fails
    /// for any reason (headless, missing libsecret, keyring locked,
    /// user denied access), falls back transparently to the HKDF path.
    ///
    /// `service_name` scopes the key inside the OS keyring — use a
    /// constant per app. `storage_dir` is where the install UUID (salt
    /// for the KDF fallback) lives; it must be writable and persistent
    /// across app restarts.
    pub fn open(service_name: &str, storage_dir: &Path) -> Result<Self, SecretError> {
        let backend = Backend::new(service_name, storage_dir)?;
        Ok(Self {
            backend: std::sync::Arc::new(backend),
        })
    }

    /// Construct directly from a provided backend, useful for tests.
    #[doc(hidden)]
    pub fn from_backend(backend: Backend) -> Self {
        Self {
            backend: std::sync::Arc::new(backend),
        }
    }

    /// Wrap a secret for at-rest storage. The returned bytes are
    /// self-describing (include backend marker, nonce, ciphertext+tag)
    /// so [`unwrap`](Self::unwrap) on the same machine can round-trip.
    pub fn wrap(&self, plaintext: &[u8]) -> Result<Vec<u8>, SecretError> {
        self.backend.wrap(plaintext)
    }

    /// Unwrap previously [`wrap`](Self::wrap)-produced bytes.
    pub fn unwrap(&self, wrapped: &[u8]) -> Result<Vec<u8>, SecretError> {
        self.backend.unwrap(wrapped)
    }

    /// Which backend is actually in use. Exposed for diagnostics /
    /// settings UI ("Offline cache secured by OS keyring" vs "…by
    /// device-derived key").
    pub fn backend_kind(&self) -> BackendKind {
        self.backend.kind()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_vault() -> (SecretBox, TempDir) {
        let dir = TempDir::new().expect("tempdir");
        // Use a service name that's extremely unlikely to collide with a
        // real keyring entry on the dev machine. The KDF fallback path is
        // exercised by design because the sandboxed tempdir + nonexistent
        // entry guarantees a fresh state; the keyring may be reachable
        // but we accept either backend — the round-trip still holds.
        let vault = SecretBox::open("qbz-secrets-test-harness", dir.path())
            .expect("open vault");
        (vault, dir)
    }

    #[test]
    fn roundtrip_small_secret() {
        let (vault, _dir) = test_vault();
        let payload = b"hello, secret";
        let wrapped = vault.wrap(payload).expect("wrap");
        let unwrapped = vault.unwrap(&wrapped).expect("unwrap");
        assert_eq!(unwrapped, payload);
    }

    #[test]
    fn roundtrip_16_byte_content_key() {
        // The exact shape of a CMAF content key — the primary use case.
        let (vault, _dir) = test_vault();
        let key = [0x42u8; 16];
        let wrapped = vault.wrap(&key).expect("wrap");
        let unwrapped = vault.unwrap(&wrapped).expect("unwrap");
        assert_eq!(unwrapped, &key[..]);
    }

    #[test]
    fn tampering_is_detected() {
        let (vault, _dir) = test_vault();
        let mut wrapped = vault.wrap(b"important data").expect("wrap");
        // Flip one bit in the ciphertext region (past the 14-byte header)
        wrapped[20] ^= 0x01;
        let result = vault.unwrap(&wrapped);
        assert!(result.is_err(), "GCM tag must detect tampering");
    }

    #[test]
    fn two_wraps_of_same_plaintext_differ() {
        // Nonce is random per wrap — two calls must produce distinct ciphertext
        let (vault, _dir) = test_vault();
        let a = vault.wrap(b"same input").expect("wrap");
        let b = vault.wrap(b"same input").expect("wrap");
        assert_ne!(a, b);
    }
}
