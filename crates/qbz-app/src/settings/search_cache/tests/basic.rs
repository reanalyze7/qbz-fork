use super::fixtures::{album, artist, playlist, results, track, unique_test_dir};
use crate::settings::search_cache::{normalize_query, SearchCache};

// (a) put-then-get round-trips albums/tracks/playlists/artists.
#[test]
fn put_then_get_roundtrips_all_categories() {
    let dir = unique_test_dir("search-roundtrip");
    let mut cache = SearchCache::new(&dir);

    let r = results(
        vec![album(1), album(2)],
        vec![track(10)],
        vec![artist(100), artist(200)],
        vec![playlist(7)],
    );
    cache.put("Pink Floyd", &r);

    let got = cache.get("Pink Floyd").expect("cached entry");
    assert_eq!(got.albums.items.iter().map(|a| a.id.clone()).collect::<Vec<_>>(), vec!["1", "2"]);
    assert_eq!(got.albums.total, 2);
    assert_eq!(got.albums.offset, 0);
    assert_eq!(got.albums.limit, 2);
    assert_eq!(got.tracks.items.iter().map(|t| t.id).collect::<Vec<_>>(), vec![10]);
    assert_eq!(got.artists.items.iter().map(|a| a.id).collect::<Vec<_>>(), vec![100, 200]);
    assert_eq!(got.playlists.items.iter().map(|p| p.id).collect::<Vec<_>>(), vec![7]);
    assert!(got.most_popular.is_none());

    // Unknown key -> None.
    assert!(cache.get("nothing here").is_none());

    let _ = std::fs::remove_dir_all(dir);
}

// (d) normalize_query collapses whitespace/case.
#[test]
fn normalize_collapses_whitespace_and_case() {
    assert_eq!(normalize_query("  Pink   Floyd  "), "pink floyd");
    assert_eq!(normalize_query("MILES\tDAVIS"), "miles davis");
    assert_eq!(normalize_query("a\n\n b"), "a b");
    assert_eq!(normalize_query("Already normal"), "already normal");
    assert_eq!(normalize_query(""), "");

    // The cache keys equivalently-normalized queries together.
    let dir = unique_test_dir("search-normkey");
    let mut cache = SearchCache::new(&dir);
    cache.put("  Pink   Floyd ", &results(vec![album(1)], vec![], vec![], vec![]));
    assert!(cache.get("pink floyd").is_some());
    assert!(cache.get("PINK FLOYD").is_some());
    let _ = std::fs::remove_dir_all(dir);
}
