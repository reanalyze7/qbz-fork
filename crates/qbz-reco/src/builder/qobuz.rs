//! Qobuz similar-artist fetching + mapping into a partial vector.
//!
//! Locking discipline: the `qobuz_client` read guard is scoped to the block
//! that performs the network call and is dropped before the `store` lock is
//! taken (no guard crosses an `.await`).

use super::ArtistVectorBuilder;
use crate::sparse_vector::SparseVector;

impl ArtistVectorBuilder {
    /// Build vector component from Qobuz similar artists
    pub(super) async fn build_from_qobuz(
        &self,
        qobuz_artist_id: u64,
    ) -> Result<(SparseVector, usize), String> {
        let similar = {
            let guard__ = self.qobuz_client.read().await;
            let client = guard__
                .as_ref()
                .ok_or("No active session - please log in")?;
            client
                .get_similar_artists(qobuz_artist_id, 20, 0)
                .await
                .map_err(|e| format!("Qobuz API error: {}", e))?
        };

        let mut vector = SparseVector::new();
        let mut count = 0;
        let mut guard__ = self.store.lock().await;
        let store = guard__
            .as_mut()
            .ok_or("No active session - please log in")?;

        for artist in similar.items {
            // Resolve Qobuz artist to a synthetic MBID node based on Qobuz ID.
            let synthetic_mbid = format!("qobuz:{}", artist.id);

            let idx = store.get_or_create_idx(&synthetic_mbid, Some(&artist.name))?;
            let weight = self.weights.qobuz_similar;
            vector.set(idx, weight);
            count += 1;
        }

        Ok((vector, count))
    }
}
