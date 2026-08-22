//! Network-mount detection for local library paths.
//!
//! On Linux, reads /proc/mounts (or /run/host/proc/mounts as a fallback
//! for sandboxed apps like Flatpak / Snap) and classifies a given
//! filesystem path by the fs type of its longest-matching mount point.
//!
//! The UI consumes the resulting is_network_mount flag to mark tracks
//! as unreachable when the user is under forced offline mode (cable
//! unplugged / ISP down). In that state a path that still reads
//! /home/user/music can be sitting on a CIFS share or SSHFS — the
//! heuristic the frontend originally used (string-match /mnt, /media)
//! misses those cases entirely, especially inside sandboxes where the
//! user's music folder is commonly bind-mounted from an SMB share.

use std::path::Path;

#[cfg(target_os = "linux")]
mod classify;
#[cfg(target_os = "linux")]
mod io;
#[cfg(target_os = "linux")]
mod parse;
#[cfg(all(test, target_os = "linux"))]
mod tests;

#[cfg(target_os = "linux")]
use classify::{best_fs_type, is_network_fs, normalize_network_label};
#[cfg(target_os = "linux")]
use io::read_mounts;

/// Return true when `path` lives on a network-backed filesystem.
///
/// Non-Linux platforms fall through to `false` — we don't have a
/// portable story for macOS / Windows yet. The frontend still has a
/// defensive string-match heuristic for UNC paths and common mount
/// prefixes, which picks up the easy cases on those platforms.
#[cfg(target_os = "linux")]
pub fn is_network_path(path: &Path) -> bool {
    let mounts = read_mounts();
    if mounts.is_empty() {
        return false;
    }

    // Canonicalize for best matching; fall back to raw path if the
    // file already disappeared / permission denied.
    let target = path
        .canonicalize()
        .unwrap_or_else(|_| path.to_path_buf());

    match best_fs_type(&mounts, &target.to_string_lossy()) {
        Some(fs_type) => is_network_fs(fs_type),
        None => false,
    }
}

#[cfg(not(target_os = "linux"))]
pub fn is_network_path(_path: &Path) -> bool {
    false
}

/// Return the normalized network-filesystem label (`cifs` / `nfs` / `sshfs` /
/// `rclone` / `webdav` / `glusterfs` / `ceph` / `other`) for `path` when it
/// lives on a network-backed filesystem, else `None`. Mirrors the fs-type
/// classification the Tauri side persisted via `crate::network::is_network_path`,
/// so the Slint folder-settings modal can show + store the same auto-detected
/// type. (`is_network_path` returns only the bool; this adds the label.)
#[cfg(target_os = "linux")]
pub fn network_fs_label(path: &Path) -> Option<String> {
    let mounts = read_mounts();
    if mounts.is_empty() {
        return None;
    }
    let target = path
        .canonicalize()
        .unwrap_or_else(|_| path.to_path_buf());

    let fs_type = best_fs_type(&mounts, &target.to_string_lossy())?;
    if !is_network_fs(fs_type) {
        return None;
    }
    Some(normalize_network_label(fs_type))
}

#[cfg(not(target_os = "linux"))]
pub fn network_fs_label(_path: &Path) -> Option<String> {
    None
}
