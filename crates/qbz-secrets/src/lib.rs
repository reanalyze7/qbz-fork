//! Device-bound secret wrapping for QBZ.
//!
//! This crate exists to give QBZ a single, uniform way to wrap small
//! secrets (AES keys, session tokens, OAuth refresh material) so they
//! can be persisted to disk without giving an attacker with filesystem
//! access everything they need to use those secrets on another machine.
//!
//! # Why
//!
//! The immediate consumer is the offline music cache. Qobuz streams each
//! track with CMAF, where a per-track AES key is derived at runtime from
//! session material. To make "save for offline" work we must store each
//! track's content key somewhere. Storing it plaintext in SQLite next to
//! the encrypted audio files would defeat the purpose — copying the DB +
//! files is enough to play them anywhere. So we wrap the key.
//!
//! # Design
//!
//! Two backends, same API:
//!
//! 1. **OS keyring** (preferred): the master AES-256 key lives inside the
//!    OS secure store — libsecret/gnome-keyring on Linux, Keychain on
//!    macOS, DPAPI on Windows. This is the "gold standard" device binding
//!    used by commercial music apps.
//! 2. **KDF fallback** (headless): when the OS keyring is not reachable
//!    (typical on Raspberry Pi / server / Docker), the master key is
//!    derived on the fly via HKDF-SHA256 over `machine-id` + a persistent
//!    per-install UUID + a constant salt. This is weaker than the keyring
//!    path because anyone with filesystem access to `machine-id` and the
//!    install directory can reconstruct the key, but it still defeats
//!    naive copy-paste attacks across machines and it lets the daemon
//!    variant of QBZ work without a desktop session behind it.
//!
//! Both backends produce and consume the same on-disk [`WrappedSecret`]
//! envelope, so the caller never needs to care which path was used.
//!
//! # Threat model
//!
//! What this protects against:
//!
//! - Copying the offline cache directory (or the whole SQLite DB) to
//!   another machine. On the new machine the keyring entry is missing
//!   (or machine-id differs), so the wrapped keys can't be unwrapped.
//! - Casual inspection of the DB. Keys are not recoverable by reading
//!   the wrapped blob alone.
//!
//! What this does **not** protect against:
//!
//! - A determined attacker on the same machine with shell access and
//!   the ability to invoke QBZ: they can always ask QBZ itself to
//!   decrypt, which is the right property (same threat model as every
//!   local DRM).
//! - Someone re-compiling QBZ with telemetry on the decrypted bytes —
//!   that's a source-modification attack, not a cryptographic one, and
//!   it's outside the scope of at-rest wrapping.
//!
//! # API
//!
//! ```rust,ignore
//! use qbz_secrets::SecretBox;
//!
//! // Open once per app, at startup. service_name scopes the key inside
//! // the OS keyring and is part of the HKDF salt for the fallback path.
//! let vault = SecretBox::open("qbz", storage_dir).await?;
//!
//! // Wrap a content key before persisting to SQLite
//! let wrapped: Vec<u8> = vault.wrap(&content_key_bytes)?;
//!
//! // Unwrap when reading it back
//! let original: Vec<u8> = vault.unwrap(&wrapped)?;
//! ```

mod backend;
mod cipher;
mod envelope;
mod error;
mod install_id;
mod secret_box;

pub use backend::{Backend, BackendKind};
pub use envelope::WrappedSecret;
pub use error::SecretError;
pub use secret_box::SecretBox;
