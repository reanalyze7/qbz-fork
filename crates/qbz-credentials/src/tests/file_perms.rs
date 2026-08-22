use std::fs;

use crate::private_file::write_private_file;

#[cfg(unix)]
#[test]
fn write_private_file_is_owner_rw_only() {
    use std::os::unix::fs::PermissionsExt;
    let dir = std::env::temp_dir().join(format!("qbz-cred-mode-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("secret.bin");
    write_private_file(&path, b"secret-bytes").unwrap();
    let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600, "expected 0o600, got {mode:o}");
    let _ = fs::remove_dir_all(&dir);
}

#[cfg(unix)]
#[test]
fn tighten_private_file_mode_fixes_loose_existing_files() {
    use crate::private_file::tighten_private_file_mode;
    use std::os::unix::fs::PermissionsExt;
    let dir = std::env::temp_dir().join(format!("qbz-cred-tighten-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).unwrap();
    let path = dir.join("legacy-secret");
    fs::write(&path, b"legacy-bytes").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();

    tighten_private_file_mode(&path);

    let file_mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
    assert_eq!(file_mode, 0o600, "expected 0o600, got {file_mode:o}");
    let dir_mode = fs::metadata(&dir).unwrap().permissions().mode() & 0o777;
    assert_eq!(dir_mode, 0o700, "expected 0o700, got {dir_mode:o}");
    let _ = fs::remove_dir_all(&dir);
}
