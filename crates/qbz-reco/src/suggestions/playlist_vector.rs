//! Combined playlist vector computation (step 2 of `generate_suggestions`).

use super::SuggestionsEngine;
use crate::sparse_vector::SparseVector;

impl SuggestionsEngine {
    /// Compute combined vector for playlist artists
    pub(super) async fn compute_playlist_vector(
        &self,
        artist_mbids: &[String],
    ) -> Result<SparseVector, String> {
        let mut combined = SparseVector::new();
        let guard__ = self.store.lock().await;
        let store = guard__
            .as_ref()
            .ok_or("No active session - please log in")?;

        for mbid in artist_mbids {
            if let Some(vector) = store.get_vector(mbid) {
                combined = combined.add(&vector);
            }
        }

        // Normalize to unit length
        Ok(combined.normalize())
    }
}
