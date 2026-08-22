//! In-memory TTL cache entries and cache-management methods for `SongLinkClient`.

use std::time::Instant;

use super::SongLinkClient;
use crate::odesli::simplified::SongLinkResponse;
use crate::odesli::CACHE_TTL;

/// Cached entry with TTL
pub(super) struct CacheEntry {
    pub(super) response: SongLinkResponse,
    pub(super) created_at: Instant,
}

impl CacheEntry {
    fn is_expired(&self) -> bool {
        self.created_at.elapsed() > CACHE_TTL
    }
}

impl SongLinkClient {
    /// Get from cache if not expired
    pub(super) fn get_from_cache(&self, key: &str) -> Option<SongLinkResponse> {
        let cache = self.cache.lock().ok()?;
        let entry = cache.get(key)?;

        if entry.is_expired() {
            None
        } else {
            Some(entry.response.clone())
        }
    }

    /// Store in cache
    pub(super) fn store_in_cache(&self, key: String, response: SongLinkResponse) {
        if let Ok(mut cache) = self.cache.lock() {
            // Clean up expired entries occasionally
            if cache.len() > 100 {
                cache.retain(|_, entry| !entry.is_expired());
            }

            cache.insert(
                key,
                CacheEntry {
                    response,
                    created_at: Instant::now(),
                },
            );
        }
    }

    /// Clear the cache
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear();
        }
    }
}
