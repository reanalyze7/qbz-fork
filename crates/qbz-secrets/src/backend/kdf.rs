use std::path::Path;

use hkdf::Hkdf;
use sha2::Sha256;

use crate::error::SecretError;
use crate::install_id;

use super::{HKDF_INFO, MASTER_KEY_LEN};

pub(super) fn derive_fallback_key(
    service_name: &str,
    storage_dir: &Path,
) -> Result<[u8; MASTER_KEY_LEN], SecretError> {
    // Assemble salt inputs. Any component can be missing; we still
    // derive a usable key as long as at least one is present (the
    // install UUID is guaranteed after first run).
    let install_uuid = install_id::load_or_create(storage_dir).map_err(SecretError::Io)?;
    let machine = install_id::machine_id().unwrap_or_default();

    // The "IKM" is the concatenation of service name + machine id +
    // install uuid. None of these are secrets; the security comes from
    // the fact that all three need to be present on the same filesystem
    // to reconstruct them.
    let mut ikm: Vec<u8> = Vec::with_capacity(service_name.len() + machine.len() + 64);
    ikm.extend_from_slice(service_name.as_bytes());
    ikm.push(0);
    ikm.extend_from_slice(machine.as_bytes());
    ikm.push(0);
    ikm.extend_from_slice(install_uuid.as_bytes());

    // HKDF-SHA256 with a fixed salt (info carries the version).
    let hk = Hkdf::<Sha256>::new(None, &ikm);
    let mut out = [0u8; MASTER_KEY_LEN];
    hk.expand(HKDF_INFO, &mut out)
        .map_err(|e| SecretError::Other(format!("HKDF expand: {}", e)))?;

    log::info!(
        "[qbz-secrets] Derived 256-bit master key via HKDF (machine-id-present={})",
        !machine.is_empty()
    );
    Ok(out)
}
