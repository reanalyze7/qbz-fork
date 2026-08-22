//! Vector builder - constructs artist vectors from MusicBrainz and Qobuz data
//!
//! Integrates with MusicBrainz for relationship data and Qobuz for similarity
//! data to build sparse vectors for each artist. Ported 1:1 from the Tauri
//! `artist_vectors::builder`, keeping its `Arc<Mutex>` ownership: the
//! MusicBrainz cache holds a `!Sync` rusqlite `Connection`, so the clients +
//! store are held behind Send+Sync handles and locked only for short,
//! await-free windows — otherwise the suggestions future (which runs on a
//! spawned task) would not be `Send`. The dead `resolve_qobuz_to_mbid` helper
//! (epic D3) is dropped. Only the Qobuz client type is swapped to
//! `qbz_qobuz::QobuzClient`; the MusicBrainz types already live in
//! `qbz_integrations`.
//!
//! **Locking discipline (critical, preserve across all submodules)**: every
//! guard (`store.lock().await`, `mb_cache.lock()`, `qobuz_client.read().await`)
//! is scoped in a block and dropped before crossing an `.await` — required for
//! the suggestions future to remain `Send`. Never hold a lock guard across an
//! `.await` point.

mod ensure;
mod musicbrainz;
mod qobuz;
#[cfg(test)]
mod tests;
mod types;

pub use types::{ArtistVectorBuilder, BuildResult};

use crate::sparse_vector::SparseVector;

impl ArtistVectorBuilder {
    /// Build a vector for an artist, fetching data from all sources.
    ///
    /// 1. Fetches MusicBrainz relationships (members, groups, collaborators)
    /// 2. Fetches Qobuz similar artists (if Qobuz ID available)
    /// 3. Combines all data into a weighted sparse vector
    /// 4. Persists the vector to the store
    pub async fn build_vector(
        &self,
        artist_mbid: &str,
        artist_name: Option<&str>,
        qobuz_artist_id: Option<u64>,
    ) -> Result<BuildResult, String> {
        let mut vector = SparseVector::new();
        let mut sources = Vec::new();
        let mut mb_relations_count = 0;
        let mut qobuz_similar_count = 0;

        // Store vectors for later persistence (to avoid holding the store lock
        // across the network awaits).
        let mut mb_vec_to_store: Option<SparseVector> = None;
        let mut qobuz_vec_to_store: Option<SparseVector> = None;

        // 1. Get or create index for this artist
        {
            let mut guard__ = self.store.lock().await;
            let store = guard__
                .as_mut()
                .ok_or("No active session - please log in")?;
            store.get_or_create_idx(artist_mbid, artist_name)?;
        }

        // 2. Fetch MusicBrainz relationships
        match self.build_from_musicbrainz(artist_mbid).await {
            Ok((mb_vec, count)) => {
                vector = vector.add(&mb_vec);
                mb_relations_count = count;
                if count > 0 {
                    sources.push("musicbrainz".to_string());
                    mb_vec_to_store = Some(mb_vec);
                }
            }
            Err(e) => {
                log::warn!(
                    "Failed to fetch MusicBrainz relations for {}: {}",
                    artist_mbid,
                    e
                );
            }
        }

        // 3. Fetch Qobuz similar artists (if we have the ID)
        if let Some(qobuz_id) = qobuz_artist_id {
            match self.build_from_qobuz(qobuz_id).await {
                Ok((qobuz_vec, count)) => {
                    vector = vector.add(&qobuz_vec);
                    qobuz_similar_count = count;
                    if count > 0 {
                        sources.push("qobuz".to_string());
                        qobuz_vec_to_store = Some(qobuz_vec);
                    }
                }
                Err(e) => {
                    log::warn!("Failed to fetch Qobuz similar for {}: {}", qobuz_id, e);
                }
            }
        }

        // 4. Persist the vectors (using saved vectors to avoid holding the lock
        // across the awaits above).
        {
            let mut guard__ = self.store.lock().await;
            let store = guard__
                .as_mut()
                .ok_or("No active session - please log in")?;

            if let Some(mb_vec) = mb_vec_to_store {
                store.set_vector(artist_mbid, &mb_vec, "musicbrainz")?;
            }

            if let Some(qobuz_vec) = qobuz_vec_to_store {
                store.set_vector(artist_mbid, &qobuz_vec, "qobuz")?;
            }
        }

        Ok(BuildResult {
            vector,
            mb_relations_count,
            qobuz_similar_count,
            sources,
        })
    }
}
