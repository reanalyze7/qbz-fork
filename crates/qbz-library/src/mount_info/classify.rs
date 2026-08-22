#![cfg(target_os = "linux")]
//! Pure matching/classification logic over an already-parsed mount table.

/// Filesystem types that require network to be reachable. Matched as
/// a prefix against the fs_type column of /proc/mounts so variants
/// like `fuse.sshfs` / `fuse.rclone` / `nfs4` all hit the same rule.
const NETWORK_FS_PREFIXES: &[&str] = &[
    "nfs",
    "cifs",
    "smb",
    "smbfs",
    "smb3",
    "fuse.sshfs",
    "fuse.rclone",
    "fuse.gvfs",
    "fuse.gvfsd",
    "fuse.davfs",
    "fuse.rclonefs",
    "davfs",
    "webdav",
    "9p",
    "ceph",
    "glusterfs",
    "afs",
    "afp",
];

/// Longest-mount-point match of `target` against the mount table, honoring
/// path-component boundaries: `/mnt/music` matches `/mnt/music` and
/// `/mnt/music/Albums/x.flac` but NOT `/mnt/music2` (the previous raw
/// `starts_with` matched the sibling too, inheriting the wrong fs type).
/// `/` is always present and any deeper mount shadows it.
pub(crate) fn best_fs_type<'a>(mounts: &'a [(String, String)], target: &str) -> Option<&'a str> {
    let mut best: Option<(&'a str, usize)> = None;
    for (mount_point, fs_type) in mounts {
        if !path_within_mount(target, mount_point) {
            continue;
        }
        let len = mount_point.len();
        if best.map(|(_, l)| l < len).unwrap_or(true) {
            best = Some((fs_type.as_str(), len));
        }
    }
    best.map(|(t, _)| t)
}

/// True when `target` IS `mount_point` or lives underneath it, on a path
/// component boundary.
fn path_within_mount(target: &str, mount_point: &str) -> bool {
    if mount_point == "/" {
        return target.starts_with('/');
    }
    match target.strip_prefix(mount_point.trim_end_matches('/')) {
        Some("") => true,
        Some(rest) => rest.starts_with('/'),
        None => false,
    }
}

/// Collapse the raw /proc/mounts fs type to the label set the folder-settings
/// modal exposes. Unknown network types fall through to `other`.
pub(crate) fn normalize_network_label(fs_type: &str) -> String {
    let lower = fs_type.to_lowercase();
    let base = lower.strip_prefix("fuse.").unwrap_or(&lower);
    match base {
        "nfs" | "nfs4" => "nfs",
        "cifs" | "smb" | "smbfs" | "smb3" => "cifs",
        "sshfs" => "sshfs",
        "rclone" | "rclonefs" => "rclone",
        "davfs" | "webdav" => "webdav",
        "glusterfs" => "glusterfs",
        "ceph" => "ceph",
        _ => "other",
    }
    .to_string()
}

pub(crate) fn is_network_fs(fs_type: &str) -> bool {
    // A prefix hits on: the exact type ("nfs", "cifs"), a dotted scheme
    // ("fuse.sshfs.x"), or a version suffix ("nfs4", "nfs3", "smb3" — pure
    // digits after the prefix). The previous `== || starts_with("{prefix}.")`
    // missed the version-suffixed forms, so `nfs4` — the fs type every
    // modern NFS mount reports in /proc/mounts — classified as LOCAL.
    NETWORK_FS_PREFIXES
        .iter()
        .any(|prefix| match fs_type.strip_prefix(prefix) {
            Some("") => true,
            Some(rest) => {
                rest.starts_with('.') || rest.chars().all(|c| c.is_ascii_digit())
            }
            None => false,
        })
}
