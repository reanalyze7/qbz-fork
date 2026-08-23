use std::path::Path;

use super::error::BundleError;
use super::types::{Bundle, ExportSource, ProfilePaths};

// ---- decrypted token load (export --include-auth) ----
pub(super) fn load_decrypted_token(
    source: &ExportSource,
    paths: &ProfilePaths,
) -> Result<Option<String>, BundleError> {
    match source {
        ExportSource::Desktop => match qbz_credentials::load_oauth_token() {
            Ok(Some(t)) => Ok(Some(t)),
            Ok(None) => {
                // Distinguish "no token" from "present but undecryptable" (IV1:
                // the portal secret is bound to the desktop session, §4.1).
                if paths.config_root.join(".qbz-oauth-token").exists() {
                    Err(BundleError::TokenDecryptFailed)
                } else {
                    Ok(None)
                }
            }
            Err(_) => {
                if paths.config_root.join(".qbz-oauth-token").exists() {
                    Err(BundleError::TokenDecryptFailed)
                } else {
                    Ok(None)
                }
            }
        },
        ExportSource::Daemon(_) => {
            match qbz_credentials::load_oauth_token_at(&paths.config_root) {
                Ok(Some(t)) => Ok(Some(t)),
                Ok(None) => Ok(None),
                Err(e) => Err(BundleError::Io(e)),
            }
        }
    }
}

/// The suggested export filename `qbz-settings-YYYYMMDD.qbzb` (04 §1).
pub fn default_filename() -> String {
    format!("qbz-settings-{}.qbzb", chrono::Utc::now().format("%Y%m%d"))
}

/// Serialize a bundle to `path`, ALWAYS mode 0600 — fail rather than fall back
/// to a wider mode (04 §6). Shared by the CLI and the P1 desktop modal.
pub fn write_bundle_file(path: &Path, bundle: &Bundle) -> Result<(), BundleError> {
    let json = bundle.to_json_string()?;
    #[cfg(unix)]
    {
        use std::io::Write as _;
        use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
        let mut opts = std::fs::OpenOptions::new();
        opts.write(true).create(true).truncate(true).mode(0o600);
        let mut f = opts
            .open(path)
            .map_err(|e| BundleError::Io(format!("could not create {}: {e}", path.display())))?;
        f.write_all(json.as_bytes())
            .map_err(|e| BundleError::Io(format!("could not write {}: {e}", path.display())))?;
        // Enforce 0600 even if the file pre-existed with a wider mode.
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).map_err(|e| {
            BundleError::Io(format!(
                "refusing to leave {} more permissive than 0600: {e}",
                path.display()
            ))
        })?;
        Ok(())
    }
    #[cfg(not(unix))]
    {
        std::fs::write(path, json.as_bytes())
            .map_err(|e| BundleError::Io(format!("could not write {}: {e}", path.display())))
    }
}
