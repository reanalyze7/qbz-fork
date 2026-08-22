//! V2 structured track cache (distinct from the legacy JSON recording cache).

use rusqlite::params;

use super::super::models::{MatchConfidence, ResolvedTrack};
use super::MusicBrainzCache;

impl MusicBrainzCache {
    /// Get cached track by ISRC (V2 structured format)
    pub fn get_track(&self, isrc: &str) -> Result<Option<ResolvedTrack>, String> {
        let result: rusqlite::Result<ResolvedTrack> = self.conn.query_row(
            "SELECT recording_mbid, title, artist_mbids, release_mbid, isrcs, confidence
             FROM resolved_tracks WHERE isrc = ?",
            [isrc],
            |row| {
                let artist_mbids_json: String = row.get(2)?;
                let isrcs_json: String = row.get(4)?;
                let confidence_str: String = row.get(5)?;
                Ok(ResolvedTrack {
                    recording_mbid: row.get(0)?,
                    title: row.get(1)?,
                    artist_mbids: serde_json::from_str(&artist_mbids_json).unwrap_or_default(),
                    release_mbid: row.get(3)?,
                    isrcs: serde_json::from_str(&isrcs_json).unwrap_or_default(),
                    confidence: match confidence_str.as_str() {
                        "exact" => MatchConfidence::Exact,
                        "high" => MatchConfidence::High,
                        "medium" => MatchConfidence::Medium,
                        "low" => MatchConfidence::Low,
                        _ => MatchConfidence::None,
                    },
                })
            },
        );
        match result {
            Ok(track) => {
                self.increment_stat("hits");
                Ok(Some(track))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                self.increment_stat("misses");
                Ok(None)
            }
            Err(e) => Err(format!("Failed to get track: {}", e)),
        }
    }

    /// Cache a resolved track (V2 structured format)
    pub fn put_track(&self, isrc: &str, track: &ResolvedTrack) -> Result<(), String> {
        let artist_mbids_json = serde_json::to_string(&track.artist_mbids).unwrap_or_default();
        let isrcs_json = serde_json::to_string(&track.isrcs).unwrap_or_default();
        let confidence = match track.confidence {
            MatchConfidence::Exact => "exact",
            MatchConfidence::High => "high",
            MatchConfidence::Medium => "medium",
            MatchConfidence::Low => "low",
            MatchConfidence::None => "none",
        };
        self.conn
            .execute(
                "INSERT OR REPLACE INTO resolved_tracks (isrc, recording_mbid, title, artist_mbids, release_mbid, isrcs, confidence)
                 VALUES (?, ?, ?, ?, ?, ?, ?)",
                params![isrc, track.recording_mbid, track.title, artist_mbids_json, track.release_mbid, isrcs_json, confidence],
            )
            .map_err(|e| format!("Failed to cache track: {}", e))?;
        Ok(())
    }

    /// Increment a cache stat counter (hits/misses). Colocated here since the
    /// V2 `get_track`/`get_artist` methods are its only callers.
    pub(super) fn increment_stat(&self, key: &str) {
        let _ = self.conn.execute(
            "UPDATE cache_stats SET value = value + 1 WHERE key = ?",
            [key],
        );
    }
}
