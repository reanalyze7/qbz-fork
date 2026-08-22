#![cfg(target_os = "linux")]
//! Pure string parsing of `/proc/mounts` lines.

pub(crate) fn parse_mounts(contents: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for line in contents.lines() {
        let mut parts = line.split_whitespace();
        let _device = parts.next();
        let mount_point = match parts.next() {
            Some(m) => m,
            None => continue,
        };
        let fs_type = match parts.next() {
            Some(t) => t,
            None => continue,
        };
        // /proc/mounts escapes spaces as \040, tabs as \011, etc.
        // Keep the raw string — starts_with on the pattern we care
        // about is unaffected, and canonicalize will bring our input
        // into the same encoding.
        out.push((mount_point.to_string(), fs_type.to_string()));
    }
    out
}
