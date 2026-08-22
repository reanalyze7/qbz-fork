//! Cache maintenance: TTL expiry sweeps, full clears, and stats.

use rusqlite::params;

use super::{
    CacheStats, MusicBrainzCache, ARTIST_TTL_SECS, METADATA_TTL_SECS, QOBUZ_VALIDATION_TTL_SECS,
    RECORDING_TTL_SECS, RELATIONS_TTL_SECS, RELEASE_TTL_SECS, SCENE_TTL_SECS,
};

impl MusicBrainzCache {
    /// Clear expired entries from all tables
    pub fn cleanup_expired(&self) -> Result<usize, String> {
        let now = Self::current_timestamp();
        let mut total_deleted = 0;

        let tables_and_ttls = [
            ("mb_recordings", RECORDING_TTL_SECS),
            ("mb_artists", ARTIST_TTL_SECS),
            ("mb_releases", RELEASE_TTL_SECS),
            ("mb_artist_relations", RELATIONS_TTL_SECS),
            ("mb_artist_metadata", METADATA_TTL_SECS),
            ("mb_scene_cache", SCENE_TTL_SECS),
            ("mb_qobuz_validation", QOBUZ_VALIDATION_TTL_SECS),
        ];

        for (table, ttl) in &tables_and_ttls {
            total_deleted += self
                .conn
                .execute(
                    &format!("DELETE FROM {} WHERE fetched_at <= ?", table),
                    params![now - ttl],
                )
                .map_err(|e| format!("Failed to cleanup {}: {}", table, e))?;
        }

        if total_deleted > 0 {
            log::info!(
                "MusicBrainz cache cleanup: removed {} expired entries",
                total_deleted
            );
        }
        Ok(total_deleted)
    }

    /// Clear all cached data (not settings)
    pub fn clear_all(&self) -> Result<(), String> {
        self.conn
            .execute_batch(
                "
                DELETE FROM mb_recordings;
                DELETE FROM mb_artists;
                DELETE FROM mb_releases;
                DELETE FROM mb_artist_relations;
                DELETE FROM mb_artist_metadata;
                DELETE FROM mb_scene_cache;
                DELETE FROM mb_qobuz_validation;
                DELETE FROM resolved_tracks;
                DELETE FROM resolved_artists;
                UPDATE cache_stats SET value = 0;
                ",
            )
            .map_err(|e| format!("Failed to clear MusicBrainz cache: {}", e))?;
        log::info!("MusicBrainz cache cleared");
        Ok(())
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> Result<CacheStats, String> {
        let recordings: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM mb_recordings", [], |row| row.get(0))
            .unwrap_or(0);
        let artists: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM mb_artists", [], |row| row.get(0))
            .unwrap_or(0);
        let releases: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM mb_releases", [], |row| row.get(0))
            .unwrap_or(0);
        let relations: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM mb_artist_relations", [], |row| {
                row.get(0)
            })
            .unwrap_or(0);

        Ok(CacheStats {
            recordings: recordings as u64,
            artists: artists as u64,
            releases: releases as u64,
            relations: relations as u64,
        })
    }

    /// TTL-based cleanup (V2 style)
    pub fn cleanup(&self, ttl_days: u32) -> Result<(u64, u64), String> {
        let cutoff = chrono::Utc::now().timestamp() - (ttl_days as i64 * 86400);
        let tracks_deleted =
            self.conn
                .execute("DELETE FROM resolved_tracks WHERE cached_at < ?", [cutoff])
                .map_err(|e| format!("Failed to cleanup tracks: {}", e))? as u64;
        let artists_deleted =
            self.conn
                .execute("DELETE FROM resolved_artists WHERE cached_at < ?", [cutoff])
                .map_err(|e| format!("Failed to cleanup artists: {}", e))? as u64;
        Ok((tracks_deleted, artists_deleted))
    }
}
