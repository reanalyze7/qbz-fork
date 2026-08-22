//! Positive/negative id-resolution regime (`reco_qobuz_cache` table).

use rusqlite::{params, OptionalExtension};

use super::{CacheLookup, RecoCache, FOUND_TTL_SECS, MISS_TTL_SECS};

impl RecoCache {
    /// Look up a resolution by key, honoring the per-regime TTL.
    pub fn get(&self, key: &str) -> CacheLookup {
        let row: Option<(i64, Option<String>, i64)> = self
            .conn
            .query_row(
                "SELECT found, qobuz_id, fetched_at FROM reco_qobuz_cache WHERE key = ?",
                params![key],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()
            .unwrap_or(None);

        match row {
            Some((found, qobuz_id, fetched_at)) => {
                let ttl = if found != 0 { FOUND_TTL_SECS } else { MISS_TTL_SECS };
                if Self::now() - fetched_at > ttl {
                    return CacheLookup::Miss; // expired
                }
                if found != 0 {
                    match qobuz_id {
                        Some(id) if !id.is_empty() => CacheLookup::Found(id),
                        _ => CacheLookup::Miss,
                    }
                } else {
                    CacheLookup::Negative
                }
            }
            None => CacheLookup::Miss,
        }
    }

    /// Store a resolution. `qobuz_id = None` records a negative (not-on-Qobuz).
    pub fn put(&self, key: &str, kind: &str, qobuz_id: Option<&str>) {
        let found = if qobuz_id.is_some() { 1 } else { 0 };
        let _ = self.conn.execute(
            "INSERT OR REPLACE INTO reco_qobuz_cache (key, kind, qobuz_id, found, fetched_at)
             VALUES (?, ?, ?, ?, ?)",
            params![key, kind, qobuz_id, found, Self::now()],
        );
    }
}
