//! Key derivation: installation salt, machine identifier, and the optional
//! XDG-portal session secret, combined into the AES-256 key used everywhere
//! else in this crate.

use sha2::{Digest, Sha256};
use std::path::Path;

use crate::machine_salt::{
    load_or_create_installation_salt_at, load_or_create_machine_id_fallback_at,
    machine_id_stable_source,
};
#[cfg(target_os = "linux")]
use crate::machine_salt::get_portal_secret;

/// Get machine-specific identifier for key derivation, resolving the persisted
/// random fallback (when needed) under `root`.
fn get_machine_id_at(root: &Path) -> Result<Vec<u8>, String> {
    if let Some(id) = machine_id_stable_source() {
        return Ok(id);
    }
    load_or_create_machine_id_fallback_at(root)
}

/// Whether the XDG portal secret takes part in key derivation.
///
/// The portal secret is reachable only through a session bus, so it makes the
/// derived key SESSION-DEPENDENT. That is fine for the desktop app (it always
/// runs inside a session) and it is Flatpak-safe entropy there, but it is wrong
/// for the daemon: `qbzd login` runs in the user's graphical session while
/// `qbzd run` is started by init (systemd system unit, OpenRC, runit) with no
/// session bus, so the daemon derives a DIFFERENT key and cannot read the token
/// it was just given — surfacing as a permanent "not logged in".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PortalKey {
    /// Desktop profile: mix the portal secret in when the session offers one.
    Session,
    /// Daemon profile: never — the key must be readable from a headless
    /// service, which is exactly where a portal-derived key is unavailable.
    Never,
}

/// Derive the encryption key with the machine-id fallback and installation salt
/// resolved under `root`. The KDF (machine-id + installation salt + optional
/// portal secret) is unchanged; `portal` selects whether the portal secret is
/// mixed in at all (see `PortalKey`).
pub(crate) fn derive_key_at(root: &Path, portal: PortalKey) -> Result<[u8; 32], String> {
    let machine_id = get_machine_id_at(root)?;
    let installation_salt = load_or_create_installation_salt_at(root)?;

    #[cfg(target_os = "linux")]
    let portal_secret = match portal {
        PortalKey::Session => get_portal_secret(),
        PortalKey::Never => None,
    };
    #[cfg(not(target_os = "linux"))]
    let portal_secret: Option<Vec<u8>> = {
        let _ = portal; // no portal off Linux — the policy is moot there.
        None
    };

    let mut hasher = Sha256::new();
    hasher.update(&installation_salt);
    if let Some(ref secret) = portal_secret {
        hasher.update(secret);
    }
    hasher.update(&machine_id);
    hasher.update(&installation_salt);

    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    Ok(key)
}
