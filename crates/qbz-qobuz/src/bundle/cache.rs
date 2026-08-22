use super::CachedBundle;
use std::path::PathBuf;

pub(crate) fn cache_path() -> Option<PathBuf> {
    Some(dirs::cache_dir()?.join("qbz").join("bundle_tokens.json"))
}

/// Load cached tokens if a valid cache file exists. Returns `None` on any error
/// (missing file, malformed JSON, empty fields) so the caller falls back to a
/// live fetch.
pub fn load_cached_bundle() -> Option<CachedBundle> {
    let path = cache_path()?;
    let data = std::fs::read(&path).ok()?;
    match serde_json::from_slice::<CachedBundle>(&data) {
        Ok(c) if !c.app_id.is_empty() && !c.secrets.is_empty() => Some(c),
        Ok(_) => {
            log::warn!("[Bundle] Cached tokens missing app_id/secrets, ignoring");
            None
        }
        Err(e) => {
            log::warn!("[Bundle] Failed to parse token cache: {}", e);
            None
        }
    }
}

pub(crate) fn save_cached_bundle(c: &CachedBundle) {
    let Some(path) = cache_path() else {
        log::warn!("[Bundle] No cache dir available, skipping token cache write");
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    match serde_json::to_vec_pretty(c) {
        Ok(bytes) => match std::fs::write(&path, bytes) {
            Ok(_) => log::info!("[Bundle] Cached tokens (version {})", c.bundle_version),
            Err(e) => log::warn!("[Bundle] Failed to write token cache: {}", e),
        },
        Err(e) => log::warn!("[Bundle] Failed to serialize token cache: {}", e),
    }
}

pub(crate) fn now_unix() -> i64 {
    chrono::Utc::now().timestamp()
}
