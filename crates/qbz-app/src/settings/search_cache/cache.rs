use super::artist_store::{ArtistCacheStore, VolatileSlice};
use super::normalize::normalize_query;
use super::VOLATILE_CACHE_CAPACITY;
use qbz_models::{SearchAllResults, SearchResultsPage};
use std::collections::{HashMap, VecDeque};
use std::path::Path;

/// The result cache. Combines an in-memory LRU for the volatile categories
/// (albums / tracks / playlists) with a persisted artist store. Keyed on the
/// [`normalize_query`] of the raw query string.
pub struct SearchCache {
    /// Volatile per-query slices, keyed by normalized query.
    pub(super) volatile: HashMap<String, VolatileSlice>,
    /// Insertion order of `volatile` keys (front = oldest), for LRU eviction.
    order: VecDeque<String>,
    /// Persisted artist slices.
    artists: ArtistCacheStore,
}

impl SearchCache {
    /// Open the cache rooted at `base_dir` (typically the per-user data dir).
    /// The persisted artist store is loaded; the volatile maps start empty.
    /// Never fails: a missing/corrupt artist file degrades to empty.
    pub fn new(base_dir: &Path) -> Self {
        Self {
            volatile: HashMap::new(),
            order: VecDeque::new(),
            artists: ArtistCacheStore::open_at(base_dir),
        }
    }

    /// Look up the cached merged result for `query`. Returns `None` when nothing
    /// at all is cached for the normalized key (neither artists nor volatile).
    ///
    /// When only the artist slice is cached, the album/track/playlist pages come
    /// back empty (but the result is still `Some`). When only the volatile slice
    /// is cached, the artist page is empty.
    pub fn get(&self, query: &str) -> Option<SearchAllResults> {
        let key = normalize_query(query);

        let volatile = self.volatile.get(&key);
        let cached_artists = self.artists.get(&key);

        if volatile.is_none() && cached_artists.is_none() {
            return None;
        }

        let (albums, tracks, playlists) = match volatile {
            Some(v) => (v.albums.clone(), v.tracks.clone(), v.playlists.clone()),
            None => (Vec::new(), Vec::new(), Vec::new()),
        };
        let artists = cached_artists.cloned().unwrap_or_default();

        Some(SearchAllResults {
            albums: page(albums),
            tracks: page(tracks),
            artists: page(artists),
            playlists: page(playlists),
            // most_popular is a derived hero, not cached; the controller can
            // recompute it from the live result. Cached reads return None.
            most_popular: None,
        })
    }

    /// Store `results` for `query`: the album/track/playlist items go into the
    /// volatile LRU (evicting the oldest query past the bound) and the artist
    /// items are persisted to disk. A live result always wins — any existing
    /// entry for the key is overwritten.
    pub fn put(&mut self, query: &str, results: &SearchAllResults) {
        let key = normalize_query(query);

        // --- volatile slice (LRU) ---
        let slice = VolatileSlice {
            albums: results.albums.items.clone(),
            tracks: results.tracks.items.clone(),
            playlists: results.playlists.items.clone(),
        };
        let is_new_key = !self.volatile.contains_key(&key);
        self.volatile.insert(key.clone(), slice);
        if is_new_key {
            self.order.push_back(key.clone());
        } else {
            // Refresh recency: move the key to the back (most-recent) position.
            if let Some(pos) = self.order.iter().position(|k| k == &key) {
                self.order.remove(pos);
            }
            self.order.push_back(key.clone());
        }
        self.evict_to_bound();

        // --- persisted artist slice ---
        self.artists.put(key, results.artists.items.clone());
    }

    /// Evict the oldest volatile entries until within [`VOLATILE_CACHE_CAPACITY`].
    /// Only the volatile maps are bounded; the persisted artist store is not
    /// evicted here (it is small and survives restarts by design).
    fn evict_to_bound(&mut self) {
        while self.volatile.len() > VOLATILE_CACHE_CAPACITY {
            if let Some(oldest) = self.order.pop_front() {
                self.volatile.remove(&oldest);
            } else {
                break;
            }
        }
    }
}

/// Build a `SearchResultsPage<T>` from cached items: `total` = items.len(),
/// `offset` = 0, `limit` = items.len() (a full single-page reconstruction).
pub(super) fn page<T>(items: Vec<T>) -> SearchResultsPage<T> {
    let n = items.len() as u32;
    SearchResultsPage {
        items,
        total: n,
        offset: 0,
        limit: n,
    }
}
