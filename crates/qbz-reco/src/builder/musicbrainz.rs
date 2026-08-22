//! MusicBrainz relationship fetching + mapping into a partial vector.
//!
//! Locking discipline: the `mb_cache` guard (a `std::sync::Mutex`) is scoped
//! in its own block and dropped before the `.await` on `mb_client`; the
//! `store` guard is taken only after all network awaits for this step are
//! done, so no guard crosses an `.await`.

mod extract;

use super::ArtistVectorBuilder;
use crate::sparse_vector::SparseVector;
use extract::extract_relationships;

impl ArtistVectorBuilder {
    /// Build vector component from MusicBrainz relationships
    pub(super) async fn build_from_musicbrainz(
        &self,
        artist_mbid: &str,
    ) -> Result<(SparseVector, usize), String> {
        // Try cache first
        let cached = {
            let guard__ = self
                .mb_cache
                .lock()
                .map_err(|_| "MusicBrainz cache lock poisoned")?;
            let cache = guard__
                .as_ref()
                .ok_or("No active session - please log in")?;
            cache.get_artist_relations(artist_mbid)?
        };

        let relations = if let Some(rel) = cached {
            rel
        } else {
            // Fetch from API (mb_client is Send+Sync; no lock needed)
            let response = self
                .mb_client
                .get_artist_with_relations(artist_mbid)
                .await
                .map_err(|e| e.to_string())?;

            // Extract relationships from raw response
            let extracted = extract_relationships(&response);

            // Cache it
            {
                let guard__ = self
                    .mb_cache
                    .lock()
                    .map_err(|_| "MusicBrainz cache lock poisoned")?;
                let cache = guard__
                    .as_ref()
                    .ok_or("No active session - please log in")?;
                cache.set_artist_relations(artist_mbid, &extracted)?;
            }

            extracted
        };

        let mut vector = SparseVector::new();
        let mut count = 0;

        // Get store for index lookups
        let mut guard__ = self.store.lock().await;
        let store = guard__
            .as_mut()
            .ok_or("No active session - please log in")?;

        // Process members (band → person)
        for member in &relations.members {
            let idx = store.get_or_create_idx(&member.mbid, Some(&member.name))?;
            let weight = self.weights.member_of_band;
            vector.set(idx, weight);
            count += 1;
        }

        // Process past members (slightly lower weight)
        for member in &relations.past_members {
            let idx = store.get_or_create_idx(&member.mbid, Some(&member.name))?;
            let weight = self.weights.member_of_band * 0.8; // Past members slightly less relevant
            vector.set(idx, weight);
            count += 1;
        }

        // Process groups (person → band)
        for group in &relations.groups {
            let idx = store.get_or_create_idx(&group.mbid, Some(&group.name))?;
            let weight = self.weights.has_member;
            vector.set(idx, weight);
            count += 1;
        }

        // Process collaborators
        for collab in &relations.collaborators {
            let idx = store.get_or_create_idx(&collab.mbid, Some(&collab.name))?;
            let weight = self.weights.collaboration;
            vector.set(idx, weight);
            count += 1;
        }

        Ok((vector, count))
    }
}
