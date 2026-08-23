use std::path::PathBuf;

use super::types::ProfilePaths;

// ---- desktop paths + misc ----
pub(super) fn desktop_paths() -> ProfilePaths {
    let config_root = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("qbz");
    let data_root = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("qbz");
    ProfilePaths {
        config_root,
        data_root,
    }
}

pub(super) fn hostname() -> String {
    std::env::var("HOSTNAME")
        .ok()
        .filter(|h| !h.trim().is_empty())
        .or_else(|| {
            std::fs::read_to_string("/etc/hostname")
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        })
        .unwrap_or_else(|| "unknown".to_string())
}

pub(super) fn now_rfc3339() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}
