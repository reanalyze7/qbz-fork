//! 30s-TTL memoization wrapper around `check_mount_status`, so frequent
//! polling (e.g. before each playback) doesn't hammer `canonicalize()`.

use std::sync::Mutex;
use std::time::{Duration, SystemTime};

use super::validate::check_mount_status;

struct MountStatusCache {
    path: String,
    is_mounted: bool,
    last_check: SystemTime,
}

static MOUNT_CACHE: Mutex<Option<MountStatusCache>> = Mutex::new(None);
const CACHE_DURATION: Duration = Duration::from_secs(30);

/// Check mount status with caching (30s cache)
pub fn is_offline_root_available(path: &str) -> Result<bool, String> {
    let mut cache = MOUNT_CACHE
        .lock()
        .map_err(|e| format!("Cache lock error: {}", e))?;

    if let Some(cached) = cache.as_ref() {
        if cached.path == path {
            if let Ok(elapsed) = SystemTime::now().duration_since(cached.last_check) {
                if elapsed < CACHE_DURATION {
                    return Ok(cached.is_mounted);
                }
            }
        }
    }

    // Cache miss or expired, check mount status
    let is_mounted = check_mount_status(path)?;

    *cache = Some(MountStatusCache {
        path: path.to_string(),
        is_mounted,
        last_check: SystemTime::now(),
    });

    Ok(is_mounted)
}
