//! Resolved weekly-playlist blob regime (`reco_weekly` table).

use rusqlite::{params, OptionalExtension};

use super::{RecoCache, WEEKLY_STALE_FALLBACK_SECS, WEEKLY_TTL_SECS};

impl RecoCache {
    /// Get a RESOLVED weekly playlist (JSON `Vec<TrackReco>`) by its exact
    /// `"{source_patch}:{playlist_mbid}"` key, IF still within the 9d TTL. The
    /// mbid changes weekly, so a fresh week misses here and triggers a rebuild;
    /// within the week the resolved set is served without re-paying Qobuz /
    /// MusicBrainz validation. `None` -> rebuild.
    pub fn get_weekly(&self, key: &str) -> Option<String> {
        let row: Option<(String, i64)> = self
            .conn
            .query_row(
                "SELECT data, built_at FROM reco_weekly WHERE key = ?",
                params![key],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .unwrap_or(None);
        match row {
            Some((data, built_at)) if Self::now() - built_at <= WEEKLY_TTL_SECS => Some(data),
            _ => None,
        }
    }

    /// Store a resolved weekly playlist under its `"{source_patch}:{mbid}"` key.
    /// Callers MUST only store NON-empty results so a transient empty build can
    /// never poison the row.
    pub fn put_weekly(&self, key: &str, source_patch: &str, data: &str) {
        let _ = self.conn.execute(
            "INSERT OR REPLACE INTO reco_weekly (key, source_patch, data, built_at)
             VALUES (?, ?, ?, ?)",
            params![key, source_patch, data, Self::now()],
        );
    }

    /// The most recent successfully-cached weekly for a `source_patch` (any
    /// week), within the stale-fallback window — used so a transient empty
    /// build still shows last week's row instead of nothing.
    pub fn get_latest_weekly_for_patch(&self, source_patch: &str) -> Option<String> {
        let row: Option<(String, i64)> = self
            .conn
            .query_row(
                "SELECT data, built_at FROM reco_weekly WHERE source_patch = ?
                 ORDER BY built_at DESC LIMIT 1",
                params![source_patch],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .unwrap_or(None);
        match row {
            Some((data, built_at)) if Self::now() - built_at <= WEEKLY_STALE_FALLBACK_SECS => {
                Some(data)
            }
            _ => None,
        }
    }
}
