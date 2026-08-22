//! Step 3: find related artists by summed relationship weight.
//!
//! Locking discipline: the `store` guard is scoped to this block only and
//! dropped before returning — no `.await` happens while it is held.

use super::super::SuggestionsEngine;
use crate::store::SimilarArtist;

impl SuggestionsEngine {
    pub(super) async fn find_related_artists(
        &self,
        playlist_artist_mbids: &[String],
    ) -> Result<Vec<SimilarArtist>, String> {
        let exclude_vec: Vec<String> = playlist_artist_mbids.to_vec();
        let guard__ = self.store.lock().await;
        let store = guard__
            .as_ref()
            .ok_or("No active session - please log in")?;
        // Use direct relationship lookup instead of vector similarity.
        // This finds members, collaborators, etc. from the MusicBrainz data.
        store.get_all_related_artists(playlist_artist_mbids, &exclude_vec, self.config.max_artists)
    }
}
