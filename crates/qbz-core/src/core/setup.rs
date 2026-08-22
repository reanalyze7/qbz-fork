//! Cheap state setters installed once at startup alongside auth: the D8
//! offline-only queue flag, the MusicBrainz cache handle, and the
//! per-user artist-vector store.

use qbz_integrations::musicbrainz::cache::MusicBrainzCache;
use qbz_models::FrontendAdapter;

use super::QbzCore;

impl<A: FrontendAdapter + Send + Sync + 'static> QbzCore<A> {
    /// Mark (or unmark) the current queue as built from an OFFLINE-ONLY local
    /// playlist (D8). Call right after the `set_queue` that loaded it.
    pub fn set_queue_offline_only(&self, on: bool) {
        self.queue_offline_only
            .store(on, std::sync::atomic::Ordering::Relaxed);
    }

    /// True when the current queue originates from an offline-only local
    /// playlist — QConnect must skip its cloud queue push.
    pub fn queue_is_offline_only(&self) -> bool {
        self.queue_offline_only
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Install a MusicBrainz cache. The frontend owns the data path
    /// and opens the cache; QbzCore just stores the handle and uses
    /// it transparently in `musicbrainz_get_artist_metadata` and
    /// `musicbrainz_get_artist_relationships`.
    pub fn set_musicbrainz_cache(&self, cache: MusicBrainzCache) {
        if let Ok(mut guard) = self.musicbrainz_cache.lock() {
            *guard = Some(cache);
        }
    }

    /// Install the per-user artist-vector store (playlist Suggested Songs). The
    /// frontend owns the data path and opens it via
    /// `qbz_reco::ArtistVectorStore::open_at`.
    pub async fn set_artist_vectors(&self, store: qbz_reco::ArtistVectorStore) {
        *self.artist_vectors.lock().await = Some(store);
    }

    /// Drop the per-user artist-vector store on logout.
    pub async fn clear_artist_vectors(&self) {
        *self.artist_vectors.lock().await = None;
    }
}
