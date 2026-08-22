//! Persisted installation salt / machine-id-fallback files, and the
//! XDG-portal session-secret probe. All feed into `keys::derive_key_at`.

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::RngExt;
use std::fs;
use std::path::Path;

use crate::paths::{installation_salt_path_at, machine_id_fallback_path_at};
use crate::private_file::{tighten_private_file_mode, write_private_file};

/// Load a persistent installation salt under `root`, or create one on first use.
pub(crate) fn load_or_create_installation_salt_at(root: &Path) -> Result<Vec<u8>, String> {
    let path = installation_salt_path_at(root);

    if path.exists() {
        tighten_private_file_mode(&path);
        let encoded =
            fs::read_to_string(&path).map_err(|e| format!("Failed to read salt file: {}", e))?;
        let decoded = BASE64
            .decode(encoded.trim())
            .map_err(|e| format!("Failed to decode salt file: {}", e))?;
        if decoded.len() != 32 {
            return Err("Invalid installation salt length".to_string());
        }
        return Ok(decoded);
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create salt directory: {}", e))?;
    }

    let mut salt = [0u8; 32];
    rand::rng().fill(&mut salt);
    write_private_file(&path, BASE64.encode(salt))?;

    Ok(salt.to_vec())
}

/// Load a persistent machine identifier fallback under `root`, or create one.
pub(crate) fn load_or_create_machine_id_fallback_at(root: &Path) -> Result<Vec<u8>, String> {
    let path = machine_id_fallback_path_at(root);

    if path.exists() {
        tighten_private_file_mode(&path);
        let encoded = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read machine fallback id: {}", e))?;
        let decoded = BASE64
            .decode(encoded.trim())
            .map_err(|e| format!("Failed to decode machine fallback id: {}", e))?;
        if decoded.len() != 32 {
            return Err("Invalid machine fallback id length".to_string());
        }
        return Ok(decoded);
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create machine fallback directory: {}", e))?;
    }

    let mut machine_fallback = [0u8; 32];
    rand::rng().fill(&mut machine_fallback);
    write_private_file(&path, BASE64.encode(machine_fallback))?;

    Ok(machine_fallback.to_vec())
}

/// Probe for a stable, root-independent machine identifier. Returns `None`
/// when none of `/etc/machine-id`, `$HOSTNAME`, `$USER` yields a value (the
/// caller then falls back to a persisted random id under its config root).
pub(crate) fn machine_id_stable_source() -> Option<Vec<u8>> {
    // Try /etc/machine-id first (Linux)
    if let Ok(id) = fs::read_to_string("/etc/machine-id") {
        let trimmed = id.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.as_bytes().to_vec());
        }
    }

    // Fallback to hostname
    if let Ok(hostname) = std::env::var("HOSTNAME") {
        if !hostname.trim().is_empty() {
            return Some(hostname.as_bytes().to_vec());
        }
    }

    // Last resort before the persisted fallback: use username
    if let Ok(user) = std::env::var("USER") {
        if !user.trim().is_empty() {
            return Some(user.as_bytes().to_vec());
        }
    }

    None
}

/// Retrieve per-app secret from XDG Desktop Portal (cached for session lifetime).
/// Returns None if portal is unavailable (headless, old DEs, non-Linux).
#[cfg(target_os = "linux")]
pub(crate) fn get_portal_secret() -> Option<Vec<u8>> {
    use std::sync::OnceLock;
    static PORTAL_SECRET: OnceLock<Option<Vec<u8>>> = OnceLock::new();
    PORTAL_SECRET
        .get_or_init(|| {
            let rt = tokio::runtime::Handle::try_current().ok()?;
            let (tx, rx) = std::sync::mpsc::channel();
            rt.spawn(async move {
                let _ = tx.send(ashpd::desktop::secret::retrieve().await.ok());
            });
            match rx.recv_timeout(std::time::Duration::from_secs(3)) {
                Ok(secret) => {
                    if secret.is_some() {
                        log::info!("[Credentials] Using XDG portal secret for key derivation");
                    }
                    secret
                }
                Err(_) => {
                    log::debug!("[Credentials] XDG portal secret unavailable (timeout/missing)");
                    None
                }
            }
        })
        .clone()
}
