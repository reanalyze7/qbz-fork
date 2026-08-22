//! SQLite schema definition for the MusicBrainz cache.

use super::MusicBrainzCache;

impl MusicBrainzCache {
    pub(super) fn init_schema(&self) -> Result<(), String> {
        self.conn
            .execute_batch(
                "
                -- Settings (enabled state, etc.)
                CREATE TABLE IF NOT EXISTS mb_settings (
                    key TEXT PRIMARY KEY,
                    value TEXT NOT NULL
                );

                -- Recordings indexed by ISRC
                CREATE TABLE IF NOT EXISTS mb_recordings (
                    isrc TEXT PRIMARY KEY,
                    data TEXT NOT NULL,
                    fetched_at INTEGER NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_mb_recordings_fetched ON mb_recordings(fetched_at);

                -- Artists indexed by normalized name
                CREATE TABLE IF NOT EXISTS mb_artists (
                    name_normalized TEXT PRIMARY KEY,
                    data TEXT NOT NULL,
                    fetched_at INTEGER NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_mb_artists_fetched ON mb_artists(fetched_at);

                -- Releases indexed by UPC/barcode
                CREATE TABLE IF NOT EXISTS mb_releases (
                    barcode TEXT PRIMARY KEY,
                    data TEXT NOT NULL,
                    fetched_at INTEGER NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_mb_releases_fetched ON mb_releases(fetched_at);

                -- Artist relationships indexed by MBID
                CREATE TABLE IF NOT EXISTS mb_artist_relations (
                    mbid TEXT PRIMARY KEY,
                    data TEXT NOT NULL,
                    fetched_at INTEGER NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_mb_relations_fetched ON mb_artist_relations(fetched_at);

                -- Artist metadata (location, genres, life span) indexed by MBID
                CREATE TABLE IF NOT EXISTS mb_artist_metadata (
                    mbid TEXT PRIMARY KEY,
                    data TEXT NOT NULL,
                    fetched_at INTEGER NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_mb_metadata_fetched ON mb_artist_metadata(fetched_at);

                -- Scene discovery results indexed by area + seed hash
                CREATE TABLE IF NOT EXISTS mb_scene_cache (
                    cache_key TEXT PRIMARY KEY,
                    data TEXT NOT NULL,
                    fetched_at INTEGER NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_mb_scene_fetched ON mb_scene_cache(fetched_at);

                -- Qobuz artist validation cache
                CREATE TABLE IF NOT EXISTS mb_qobuz_validation (
                    name_normalized TEXT PRIMARY KEY,
                    data TEXT NOT NULL,
                    fetched_at INTEGER NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_mb_qobuz_validation_fetched ON mb_qobuz_validation(fetched_at);

                -- V2 resolved tracks (simple cache)
                CREATE TABLE IF NOT EXISTS resolved_tracks (
                    isrc TEXT PRIMARY KEY,
                    recording_mbid TEXT NOT NULL,
                    title TEXT NOT NULL,
                    artist_mbids TEXT NOT NULL,
                    release_mbid TEXT,
                    isrcs TEXT NOT NULL,
                    confidence TEXT NOT NULL,
                    cached_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
                );

                -- V2 resolved artists (simple cache)
                CREATE TABLE IF NOT EXISTS resolved_artists (
                    name_lower TEXT PRIMARY KEY,
                    mbid TEXT NOT NULL,
                    name TEXT NOT NULL,
                    sort_name TEXT,
                    artist_type TEXT NOT NULL,
                    country TEXT,
                    disambiguation TEXT,
                    confidence TEXT NOT NULL,
                    cached_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
                );

                CREATE TABLE IF NOT EXISTS cache_stats (
                    key TEXT PRIMARY KEY,
                    value INTEGER NOT NULL DEFAULT 0
                );
                INSERT OR IGNORE INTO cache_stats (key, value) VALUES ('hits', 0);
                INSERT OR IGNORE INTO cache_stats (key, value) VALUES ('misses', 0);
            ",
            )
            .map_err(|e| format!("Failed to init MusicBrainz schema: {}", e))
    }
}
