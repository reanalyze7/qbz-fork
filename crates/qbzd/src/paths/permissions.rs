use std::path::Path;

#[cfg(unix)]
pub(super) fn ensure_config_dir(dir: &Path) {
    use std::os::unix::fs::PermissionsExt;
    match std::fs::create_dir_all(dir) {
        Ok(()) => {
            if let Err(e) = std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700)) {
                log::warn!("could not set 0700 on config dir {}: {e}", dir.display());
            }
        }
        Err(e) => {
            log::warn!("could not create config dir {}: {e}", dir.display());
        }
    }
}

#[cfg(not(unix))]
pub(super) fn ensure_config_dir(dir: &Path) {
    let _ = std::fs::create_dir_all(dir);
}
