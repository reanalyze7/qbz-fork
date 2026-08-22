//! V2 structured artist cache (distinct from the legacy JSON artist cache
//! in `artist.rs`).
//!
//! Note: unlike `artist.rs`, these methods key on `name.to_lowercase()`
//! directly rather than [`MusicBrainzCache::normalize_name`] — this is an
//! existing inconsistency, preserved as-is.

use rusqlite::params;

use super::super::models::{ArtistType, MatchConfidence, ResolvedArtist};
use super::MusicBrainzCache;

impl MusicBrainzCache {
    /// Get cached artist by name (V2 structured format)
    pub fn get_artist(&self, name: &str) -> Result<Option<ResolvedArtist>, String> {
        let name_lower = name.to_lowercase();
        let result: rusqlite::Result<ResolvedArtist> = self.conn.query_row(
            "SELECT mbid, name, sort_name, artist_type, country, disambiguation, confidence
             FROM resolved_artists WHERE name_lower = ?",
            [&name_lower],
            |row| {
                let artist_type_str: String = row.get(3)?;
                let confidence_str: String = row.get(6)?;
                Ok(ResolvedArtist {
                    mbid: row.get(0)?,
                    name: row.get(1)?,
                    sort_name: row.get(2)?,
                    artist_type: ArtistType::from(Some(artist_type_str.as_str())),
                    country: row.get(4)?,
                    disambiguation: row.get(5)?,
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
            Ok(artist) => {
                self.increment_stat("hits");
                Ok(Some(artist))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                self.increment_stat("misses");
                Ok(None)
            }
            Err(e) => Err(format!("Failed to get artist: {}", e)),
        }
    }

    /// Cache a resolved artist (V2 structured format)
    pub fn put_artist(&self, artist: &ResolvedArtist) -> Result<(), String> {
        let name_lower = artist.name.to_lowercase();
        let artist_type = match artist.artist_type {
            ArtistType::Person => "person",
            ArtistType::Group => "group",
            ArtistType::Orchestra => "orchestra",
            ArtistType::Choir => "choir",
            ArtistType::Character => "character",
            ArtistType::Other => "other",
        };
        let confidence = match artist.confidence {
            MatchConfidence::Exact => "exact",
            MatchConfidence::High => "high",
            MatchConfidence::Medium => "medium",
            MatchConfidence::Low => "low",
            MatchConfidence::None => "none",
        };
        self.conn
            .execute(
                "INSERT OR REPLACE INTO resolved_artists (name_lower, mbid, name, sort_name, artist_type, country, disambiguation, confidence)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                params![name_lower, artist.mbid, artist.name, artist.sort_name, artist_type, artist.country, artist.disambiguation, confidence],
            )
            .map_err(|e| format!("Failed to cache artist: {}", e))?;
        Ok(())
    }
}
