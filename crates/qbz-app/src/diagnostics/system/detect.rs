/// Parse /etc/os-release (or the Flatpak host equivalent) into key/value pairs.
pub(super) fn read_os_release() -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    // Try Flatpak-exposed host file first, then the normal path.
    let candidates = ["/run/host/os-release", "/etc/os-release"];
    for path in candidates {
        if let Ok(text) = std::fs::read_to_string(path) {
            for line in text.lines() {
                if let Some(idx) = line.find('=') {
                    let key = line[..idx].trim().to_string();
                    let mut value = line[idx + 1..].trim().to_string();
                    if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
                        value = value[1..value.len() - 1].to_string();
                    }
                    map.insert(key, value);
                }
            }
            if !map.is_empty() {
                return map;
            }
        }
    }
    map
}

pub(super) fn detect_kernel_version() -> Option<String> {
    std::fs::read_to_string("/proc/sys/kernel/osrelease")
        .ok()
        .map(|s| s.trim().to_string())
}

pub(super) fn detect_install_method() -> (String, Option<String>, Option<String>) {
    // (method, flatpak_runtime, flatpak_runtime_version)
    if std::env::var("FLATPAK_ID").is_ok() || std::path::Path::new("/.flatpak-info").exists() {
        let mut runtime = None;
        let mut runtime_version = None;
        if let Ok(text) = std::fs::read_to_string("/.flatpak-info") {
            for line in text.lines() {
                if let Some(value) = line.strip_prefix("runtime=") {
                    let v = value.trim();
                    if let Some(slash) = v.rfind('/') {
                        runtime = Some(v[..slash].to_string());
                        runtime_version = Some(v[slash + 1..].to_string());
                    } else {
                        runtime = Some(v.to_string());
                    }
                }
            }
        }
        return ("flatpak".to_string(), runtime, runtime_version);
    }
    if std::env::var("SNAP").is_ok() {
        return ("snap".to_string(), None, None);
    }
    if std::env::var("APPIMAGE").is_ok() {
        return ("appimage".to_string(), None, None);
    }
    if cfg!(debug_assertions) {
        return ("dev".to_string(), None, None);
    }
    ("native".to_string(), None, None)
}

/// Extract the best-available version string from the filename of a shared
/// library loaded by the current process. Looks for patterns like
/// `libfoo.so.0.15.7` → `0.15.7`, or `libfoo.so.2` → `2`.
/// Returns `None` if the library isn't mapped.
pub(super) fn detect_loaded_lib_version(lib_name_stem: &str) -> Option<String> {
    let maps = std::fs::read_to_string("/proc/self/maps").ok()?;
    let mut best: Option<String> = None;
    for line in maps.lines() {
        // Last column is the path (may contain spaces, very rare).
        let path = line.splitn(6, ' ').nth(5).unwrap_or("").trim();
        if path.is_empty() {
            continue;
        }
        let filename = std::path::Path::new(path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        if !filename.starts_with(lib_name_stem) {
            continue;
        }
        // filename example: "libwebkit2gtk-4.1.so.0.15.7"
        // Strip the "lib_name_stem.so" prefix and leading dot.
        let tail = match filename.split_once(".so") {
            Some((_, rest)) => rest.trim_start_matches('.'),
            None => continue,
        };
        if tail.is_empty() {
            continue;
        }
        // Resolve symlink target if possible — often the real file carries
        // a fuller version than the SONAME alias.
        if let Ok(real) = std::fs::canonicalize(path) {
            if let Some(real_name) = real.file_name().and_then(|s| s.to_str()) {
                if let Some((_, rest)) = real_name.split_once(".so") {
                    let real_tail = rest.trim_start_matches('.');
                    if !real_tail.is_empty() {
                        best = Some(real_tail.to_string());
                        continue;
                    }
                }
            }
        }
        best.get_or_insert_with(|| tail.to_string());
    }
    best
}
