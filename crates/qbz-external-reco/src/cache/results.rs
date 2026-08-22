//! Built-result-row blob regime (`reco_results` table).

use rusqlite::{params, OptionalExtension};

use super::RecoCache;

impl RecoCache {
    /// Get the cached BUILT result rows (JSON of `ExternalCarousels`) for `key`,
    /// IF still within `ttl_secs`. `None` -> the caller must rebuild.
    pub fn get_results(&self, key: &str, ttl_secs: i64) -> Option<String> {
        let row: Option<(String, i64)> = self
            .conn
            .query_row(
                "SELECT data, built_at FROM reco_results WHERE key = ?",
                params![key],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .unwrap_or(None);
        match row {
            Some((data, built_at)) if Self::now() - built_at <= ttl_secs => Some(data),
            _ => None,
        }
    }

    /// Store the built result rows (JSON) for `key`, stamped now.
    pub fn put_results(&self, key: &str, data: &str) {
        let _ = self.conn.execute(
            "INSERT OR REPLACE INTO reco_results (key, data, built_at) VALUES (?, ?, ?)",
            params![key, data, Self::now()],
        );
    }

    /// Drop the cached BUILT result rows for `key` (force-refresh). The next
    /// build re-populates via `put_results`.
    pub fn clear_results(&self, key: &str) {
        let _ = self
            .conn
            .execute("DELETE FROM reco_results WHERE key = ?", params![key]);
    }
}
