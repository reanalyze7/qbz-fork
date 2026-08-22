//! Steps 5-7: deduplicate, shuffle, and truncate the candidate track pool.

use super::super::{SuggestedTrack, SuggestionsEngine};
use std::collections::HashSet;

impl SuggestionsEngine {
    pub(super) fn finalize_track_pool(
        &self,
        mut all_tracks: Vec<SuggestedTrack>,
    ) -> Vec<SuggestedTrack> {
        // 5. Deduplicate by title+artist (keeps highest similarity version)
        let mut seen_titles: HashSet<String> = HashSet::new();
        all_tracks.retain(|track| {
            let key = format!(
                "{}|{}",
                track.title.to_lowercase(),
                track.artist_name.to_lowercase()
            );
            seen_titles.insert(key)
        });

        // 6. Shuffle tracks for variety (so same artist doesn't dominate)
        use rand::seq::SliceRandom;
        let mut rng = rand::rng();
        all_tracks.shuffle(&mut rng);

        // 7. Limit pool size
        all_tracks.truncate(self.config.max_pool_size);

        all_tracks
    }
}
