use super::fixtures::{album, artist, results, track, unique_test_dir};
use crate::settings::search_cache::{normalize_query, SearchCache, ARTIST_CACHE_FILE, VOLATILE_CACHE_CAPACITY};

// (b) LRU eviction drops the oldest beyond the bound.
#[test]
fn lru_evicts_oldest_beyond_bound() {
    let dir = unique_test_dir("search-lru");
    let mut cache = SearchCache::new(&dir);

    // Fill exactly to capacity with distinct volatile queries.
    for i in 0..VOLATILE_CACHE_CAPACITY {
        let q = format!("query {i}");
        cache.put(&q, &results(vec![album(i as u64)], vec![], vec![], vec![]));
    }
    // The oldest ("query 0") is still present at the bound.
    assert!(cache.volatile.contains_key(&normalize_query("query 0")));
    assert_eq!(cache.volatile.len(), VOLATILE_CACHE_CAPACITY);

    // One more distinct query evicts the oldest.
    cache.put("overflow query", &results(vec![album(999)], vec![], vec![], vec![]));
    assert_eq!(cache.volatile.len(), VOLATILE_CACHE_CAPACITY);
    assert!(!cache.volatile.contains_key(&normalize_query("query 0")));
    assert!(cache.volatile.contains_key(&normalize_query("overflow query")));

    // get() on the evicted volatile key still returns Some, because the
    // ARTIST slice persists (albums/tracks/playlists come back empty).
    let evicted = cache.get("query 0").expect("artist slice persists");
    assert!(evicted.albums.items.is_empty());

    let _ = std::fs::remove_dir_all(dir);
}

// (c) persisted artists survive a fresh SearchCache::new at the same base_dir.
#[test]
fn persisted_artists_survive_reopen() {
    let dir = unique_test_dir("search-persist");
    {
        let mut cache = SearchCache::new(&dir);
        cache.put(
            "Miles Davis",
            &results(vec![album(1)], vec![track(2)], vec![artist(42), artist(43)], vec![]),
        );
    }
    // Reopen at the same base dir: volatile is gone, artists survive.
    {
        let cache = SearchCache::new(&dir);
        let got = cache.get("Miles Davis").expect("artist slice persisted");
        assert_eq!(got.artists.items.iter().map(|a| a.id).collect::<Vec<_>>(), vec![42, 43]);
        // Volatile categories did NOT persist.
        assert!(got.albums.items.is_empty());
        assert!(got.tracks.items.is_empty());
    }
    // The on-disk file lives at <base>/search_artist_cache.json.
    assert!(dir.join(ARTIST_CACHE_FILE).exists());

    // Corrupt the file -> a fresh open degrades to empty (no panic).
    std::fs::write(dir.join(ARTIST_CACHE_FILE), "{not valid json").unwrap();
    {
        let cache = SearchCache::new(&dir);
        assert!(cache.get("Miles Davis").is_none());
    }

    let _ = std::fs::remove_dir_all(dir);
}
