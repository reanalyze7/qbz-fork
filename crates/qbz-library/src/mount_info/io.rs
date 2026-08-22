#![cfg(target_os = "linux")]
//! The only actual filesystem IO in this module: reading the mount table.

use super::parse::parse_mounts;

pub(crate) fn read_mounts() -> Vec<(String, String)> {
    // Inside Flatpak the sandbox's own /proc/mounts reflects the
    // sandbox view, which is the right lens for the app. Snap is the
    // same. Both bind-mount the host share into the sandbox, so if the
    // host mount is CIFS, the sandbox sees fuse.* or the same fs type
    // (depending on the mechanism). /run/host/proc/mounts is the
    // Flatpak escape hatch when we need the raw host view, used as a
    // fallback for cases where the sandbox doesn't expose /proc/mounts.
    for path in ["/proc/mounts", "/run/host/proc/mounts"] {
        if let Ok(contents) = std::fs::read_to_string(path) {
            return parse_mounts(&contents);
        }
    }
    Vec::new()
}
