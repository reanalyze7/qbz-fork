use super::schema::RecoStore;
use super::types::{HomeSeedLimits, HomeSeeds, TopArtistSeed};

impl RecoStore {
    // ---- Home seeds ----

    /// Gather the home/Discover ID seeds (mirrors `get_home_seeds_internal`).
    /// When trained scores exist (`reco_scores` has a `score_type='all'` row),
    /// fresh recent items are merged ahead of scored items; otherwise it falls
    /// back to the raw event-based queries.
    pub fn get_home_seeds(&self, limits: HomeSeedLimits) -> Result<HomeSeeds, String> {
        let has_scores = self.has_scores("all")?;

        let recently_played_album_ids = if has_scores {
            let recent_fresh = self.get_recent_album_ids(4)?;
            let scored = self.get_scored_album_ids("all", limits.recent_albums + 4)?;
            let merged =
                merge_unique_preserve_order(recent_fresh, scored, limits.recent_albums as usize);
            if merged.is_empty() {
                self.get_recent_album_ids(limits.recent_albums)?
            } else {
                merged
            }
        } else {
            self.get_recent_album_ids(limits.recent_albums)?
        };

        let continue_listening_track_ids = if has_scores {
            let recent_fresh = self.get_recent_track_ids(4)?;
            let scored = self.get_scored_track_ids("all", limits.continue_tracks + 4)?;
            let merged =
                merge_unique_preserve_order(recent_fresh, scored, limits.continue_tracks as usize);
            if merged.is_empty() {
                self.get_recent_track_ids(limits.continue_tracks)?
            } else {
                merged
            }
        } else {
            self.get_recent_track_ids(limits.continue_tracks)?
        };

        let top_artist_ids = if has_scores {
            let scored: Vec<TopArtistSeed> = self
                .get_scored_artist_scores("all", limits.top_artists)?
                .into_iter()
                .map(|(artist_id, score)| TopArtistSeed {
                    artist_id,
                    play_count: score.round().max(1.0) as u32,
                })
                .collect();
            if scored.is_empty() {
                self.get_top_artist_ids(limits.top_artists)?
            } else {
                scored
            }
        } else {
            self.get_top_artist_ids(limits.top_artists)?
        };

        let favorite_album_ids = if has_scores {
            let scored = self.get_scored_album_ids("favorite", limits.favorites)?;
            if scored.is_empty() {
                self.get_favorite_album_ids(limits.favorites)?
            } else {
                scored
            }
        } else {
            self.get_favorite_album_ids(limits.favorites)?
        };

        let favorite_track_ids = if has_scores {
            let scored = self.get_scored_track_ids("favorite", limits.favorites)?;
            if scored.is_empty() {
                self.get_favorite_track_ids(limits.favorites)?
            } else {
                scored
            }
        } else {
            self.get_favorite_track_ids(limits.favorites)?
        };

        Ok(HomeSeeds {
            recently_played_album_ids,
            continue_listening_track_ids,
            top_artist_ids,
            favorite_album_ids,
            favorite_track_ids,
        })
    }
}

/// Merge two lists preserving order: fresh items first, then scored items
/// (excluding duplicates) — verbatim from `helpers::merge_unique_preserve_order`.
fn merge_unique_preserve_order<T: Eq + std::hash::Hash + Clone>(
    fresh: Vec<T>,
    scored: Vec<T>,
    limit: usize,
) -> Vec<T> {
    use std::collections::HashSet;
    let mut seen: HashSet<T> = HashSet::new();
    let mut result = Vec::with_capacity(limit);
    for item in fresh {
        if seen.insert(item.clone()) {
            result.push(item);
            if result.len() >= limit {
                return result;
            }
        }
    }
    for item in scored {
        if seen.insert(item.clone()) {
            result.push(item);
            if result.len() >= limit {
                return result;
            }
        }
    }
    result
}
