//! Backend selection and runtime dispatch.
//!
//! At open time we try to use the OS keyring first. Success = we store
//! (or rotate-in) a 32-byte master key inside the keyring and use it for
//! AES-256-GCM wraps. Failure (for any reason) = we fall back to HKDF
//! over device identifiers.
//!
//! The backend discriminator is baked into every wrapped blob so a blob
//! produced by one backend can't be silently decrypted by another (and
//! so that if a user later gains keyring access, we don't accidentally
//! ignore their existing KDF-wrapped data).

mod backend_impl;
mod kdf;
mod keyring;
mod kind;

pub use backend_impl::Backend;
pub use kind::BackendKind;

const MASTER_KEY_LEN: usize = 32;
const KEYRING_ENTRY_NAME: &str = "master-key-v1";
const HKDF_INFO: &[u8] = b"qbz-secrets master-key derivation v1";

// On-disk format markers baked into every wrapped blob. These values are
// a durable ON-DISK format and must never change, regardless of which
// file declares them.
const BACKEND_MARKER_KEYRING: u8 = 0;
const BACKEND_MARKER_KDF: u8 = 1;
