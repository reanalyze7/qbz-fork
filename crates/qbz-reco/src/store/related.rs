//! Related-artist queries: single-source and multi-source aggregation.

use super::{ArtistVectorStore, SimilarArtist};
use rusqlite::params;
use std::collections::HashMap;

impl ArtistVectorStore {
    /// Get related artists for a given artist (from their vector entries).
    ///
    /// Returns artists the given artist is connected to via MusicBrainz
    /// relationships, ranked by summed weight (the production ranking).
    pub fn get_related_artists(&self, mbid: &str) -> Result<Vec<SimilarArtist>, String> {
        let Some(artist_idx) = self.get_idx(mbid) else {
            return Ok(Vec::new());
        };

        let mut stmt = self
            .conn
            .prepare(
                "SELECT target_idx, SUM(weight) as total_weight
                 FROM vector_entries
                 WHERE artist_idx = ?1
                 GROUP BY target_idx
                 ORDER BY total_weight DESC",
            )
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let rows = stmt
            .query_map(params![artist_idx], |row| {
                Ok((row.get::<_, u32>(0)?, row.get::<_, f32>(1)?))
            })
            .map_err(|e| format!("Failed to query relations: {}", e))?;

        let mut results = Vec::new();
        for row in rows.flatten() {
            let (target_idx, weight) = row;
            if let Some(target_mbid) = self.get_mbid(target_idx) {
                let name = self.get_artist_name(target_mbid);
                results.push(SimilarArtist {
                    mbid: target_mbid.to_string(),
                    name,
                    similarity: weight,
                });
            }
        }

        Ok(results)
    }

    /// Get all related artists for multiple source artists, excluding specified
    /// MBIDs. Returns a deduplicated list sorted by total weight across sources.
    pub fn get_all_related_artists(
        &self,
        source_mbids: &[String],
        exclude_mbids: &[String],
        limit: usize,
    ) -> Result<Vec<SimilarArtist>, String> {
        let exclude_set: std::collections::HashSet<_> = exclude_mbids.iter().collect();
        let mut artist_weights: HashMap<String, (Option<String>, f32)> = HashMap::new();

        for mbid in source_mbids {
            let related = self.get_related_artists(mbid)?;
            for artist in related {
                if exclude_set.contains(&artist.mbid) {
                    continue;
                }
                let entry = artist_weights
                    .entry(artist.mbid.clone())
                    .or_insert((artist.name.clone(), 0.0));
                entry.1 += artist.similarity;
            }
        }

        let mut results: Vec<_> = artist_weights
            .into_iter()
            .map(|(mbid, (name, weight))| SimilarArtist {
                mbid,
                name,
                similarity: weight,
            })
            .collect();

        results.sort_by(|a, b| {
            b.similarity
                .partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(limit);

        Ok(results)
    }
}
