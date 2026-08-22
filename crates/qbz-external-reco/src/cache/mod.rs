//! Resolved-recommendation -> Qobuz-id cache (per-user SQLite, WAL per ADR-002).
//!
//! Mirrors the `MusicBrainzCache` shape. Caches BOTH positive hits (a resolved
//! Qobuz id, TTL 30d) AND negative hits (a recommendation that does not exist on
//! Qobuz, TTL 7d) so an unfindable rec does not re-hammer the Qobuz search API
//! on every render. The connection is `!Sync`; wrap it in a `Mutex` for
//! concurrent validation (locks are brief and never held across `.await`).

use rusqlite::Connection;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

mod resolutions;
mod results;
#[cfg(test)]
mod tests;
mod weekly;

/// TTL for positive (found) entries — 30 days.
const FOUND_TTL_SECS: i64 = 30 * 24 * 60 * 60;
/// TTL for negative (not-on-Qobuz) entries — 7 days.
const MISS_TTL_SECS: i64 = 7 * 24 * 60 * 60;
/// Default TTL for the cached BUILT result rows — 48 hours (fast opens +
/// rotation: the tab paints instantly from cache within the window, and rebuilds
/// every 48h so the content is never "eternally the same"). The effective TTL is
/// caller-configurable (Recommendations cache-window setting); this is the
/// fallback when no preference is supplied.
pub const DEFAULT_RESULTS_TTL_SECS: i64 = 48 * 60 * 60;
/// TTL for a RESOLVED weekly playlist, keyed by its ListenBrainz playlist mbid —
/// 9 days (a week + slack). ListenBrainz regenerates Weekly Exploration / Weekly
/// Jams every Monday with a NEW mbid, so a new week is a natural cache miss while
/// the current week is served instantly. This is deliberately SEPARATE from the
/// 48h results blob: the weeklies have their own ListenBrainz cadence (a date per
/// playlist) and must not be clobbered by the unrelated 48h rotation.
const WEEKLY_TTL_SECS: i64 = 9 * 24 * 60 * 60;
/// How long a successfully-resolved weekly stays usable as a STALE FALLBACK (any
/// week, newest first) when the current build comes back empty — 21 days. Better
/// to show last week's row than an empty one on a transient ListenBrainz/Qobuz
/// hiccup.
const WEEKLY_STALE_FALLBACK_SECS: i64 = 21 * 24 * 60 * 60;

/// A cache lookup outcome.
pub enum CacheLookup {
    /// Resolved Qobuz id (track id as decimal string, or album id verbatim).
    Found(String),
    /// Previously resolved to "does not exist on Qobuz".
    Negative,
    /// Not cached (or expired) — caller must resolve live.
    Miss,
}

pub struct RecoCache {
    conn: Connection,
}

impl RecoCache {
    /// Open (or create) the cache at `<base_dir>/external_reco_cache.db`.
    pub fn open_at(base_dir: &Path) -> Result<Self, String> {
        std::fs::create_dir_all(base_dir)
            .map_err(|e| format!("Failed to create reco cache dir: {}", e))?;
        let conn = Connection::open(base_dir.join("external_reco_cache.db"))
            .map_err(|e| format!("Failed to open external reco cache: {}", e))?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
            .map_err(|e| format!("Failed to enable WAL: {}", e))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS reco_qobuz_cache (
                key TEXT PRIMARY KEY,
                kind TEXT NOT NULL,
                qobuz_id TEXT,
                found INTEGER NOT NULL,
                fetched_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_reco_qobuz_cache_fetched
                ON reco_qobuz_cache(fetched_at);
            CREATE TABLE IF NOT EXISTS reco_results (
                key TEXT PRIMARY KEY,
                data TEXT NOT NULL,
                built_at INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS reco_weekly (
                key TEXT PRIMARY KEY,
                source_patch TEXT NOT NULL,
                data TEXT NOT NULL,
                built_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_reco_weekly_patch
                ON reco_weekly(source_patch, built_at);",
        )
        .map_err(|e| format!("Failed to init reco cache schema: {}", e))?;
        Ok(Self { conn })
    }

    fn now() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0)
    }

    /// Drop expired rows (both regimes). Safe to call opportunistically.
    pub fn cleanup_expired(&self) -> usize {
        let now = Self::now();
        let found = self
            .conn
            .execute(
                "DELETE FROM reco_qobuz_cache WHERE found = 1 AND fetched_at <= ?",
                rusqlite::params![now - FOUND_TTL_SECS],
            )
            .unwrap_or(0);
        let miss = self
            .conn
            .execute(
                "DELETE FROM reco_qobuz_cache WHERE found = 0 AND fetched_at <= ?",
                rusqlite::params![now - MISS_TTL_SECS],
            )
            .unwrap_or(0);
        let weekly = self
            .conn
            .execute(
                "DELETE FROM reco_weekly WHERE built_at <= ?",
                rusqlite::params![now - WEEKLY_STALE_FALLBACK_SECS],
            )
            .unwrap_or(0);
        found + miss + weekly
    }
}
