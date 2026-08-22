//! Shared "write/tighten a secret file with restrictive permissions"
//! helpers used by every on-disk format in this crate.

use std::fs;
use std::io::Write;
use std::path::Path;

/// Write secret-adjacent bytes with restrictive permissions on Unix (`0o600`).
pub(crate) fn write_private_file(path: &Path, bytes: impl AsRef<[u8]>) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory {}: {e}", parent.display()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Err(e) = fs::set_permissions(parent, fs::Permissions::from_mode(0o700)) {
                log::warn!(
                    "[Credentials] Failed to tighten permissions on {}: {}",
                    parent.display(),
                    e
                );
            }
        }
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
        let mut opts = fs::OpenOptions::new();
        opts.write(true).create(true).truncate(true).mode(0o600);
        let mut f = opts
            .open(path)
            .map_err(|e| format!("Failed to open {}: {e}", path.display()))?;
        f.write_all(bytes.as_ref())
            .map_err(|e| format!("Failed to write {}: {e}", path.display()))?;
        if let Err(e) = fs::set_permissions(path, fs::Permissions::from_mode(0o600)) {
            log::warn!(
                "[Credentials] Failed to tighten permissions on {}: {}",
                path.display(),
                e
            );
        }
    }
    #[cfg(not(unix))]
    {
        fs::write(path, bytes.as_ref())
            .map_err(|e| format!("Failed to write {}: {e}", path.display()))?;
    }
    Ok(())
}

/// Tighten permissions of a pre-existing secret file on Unix.
///
/// Files created by older installs were written with the default umask and
/// can be group or world readable. They are created once and only read
/// afterwards, so `write_private_file` never gets a chance to fix them.
/// Calling this on the load paths migrates them to `0o600` (and the parent
/// config directory to `0o700`) on the next read.
#[cfg(unix)]
pub(crate) fn tighten_private_file_mode(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    if let Some(parent) = path.parent() {
        match fs::metadata(parent) {
            Ok(meta) if meta.permissions().mode() & 0o077 != 0 => {
                if let Err(e) = fs::set_permissions(parent, fs::Permissions::from_mode(0o700)) {
                    log::warn!(
                        "[Credentials] Failed to tighten permissions on {}: {}",
                        parent.display(),
                        e
                    );
                }
            }
            Ok(_) => {}
            Err(e) => {
                log::warn!(
                    "[Credentials] Failed to inspect permissions of {}: {}",
                    parent.display(),
                    e
                );
            }
        }
    }

    match fs::metadata(path) {
        Ok(meta) if meta.permissions().mode() & 0o077 != 0 => {
            if let Err(e) = fs::set_permissions(path, fs::Permissions::from_mode(0o600)) {
                log::warn!(
                    "[Credentials] Failed to tighten permissions on {}: {}",
                    path.display(),
                    e
                );
            }
        }
        Ok(_) => {}
        Err(e) => {
            log::warn!(
                "[Credentials] Failed to inspect permissions of {}: {}",
                path.display(),
                e
            );
        }
    }
}

#[cfg(not(unix))]
pub(crate) fn tighten_private_file_mode(_path: &Path) {}
