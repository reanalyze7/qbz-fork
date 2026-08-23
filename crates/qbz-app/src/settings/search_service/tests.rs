use super::{InteractionAction, SearchService};
use qbz_models::{Album, Artist, Playlist, SearchAllResults, SearchResultsPage, Track};
use std::path::PathBuf;

fn unique_test_dir(name: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "qbz-app-search-service-{name}-{}-{nonce}",
        std::process::id()
    ))
}

fn page<T>(items: Vec<T>) -> SearchResultsPage<T> {
    let n = items.len() as u32;
    SearchResultsPage {
        items,
        total: n,
        offset: 0,
        limit: n,
    }
}

fn album(id: u64) -> Album {
    serde_json::from_value(serde_json::json!({ "id": id.to_string() })).unwrap()
}
fn track(id: u64) -> Track {
    serde_json::from_value(serde_json::json!({ "id": id })).unwrap()
}
fn playlist(id: u64) -> Playlist {
    serde_json::from_value(serde_json::json!({ "id": id })).unwrap()
}
fn artist(id: u64) -> Artist {
    Artist {
        id,
        ..Default::default()
    }
}

fn sample_results() -> SearchAllResults {
    SearchAllResults {
        albums: page(vec![album(1)]),
        tracks: page(vec![track(10)]),
        artists: page(vec![artist(100)]),
        playlists: page(vec![playlist(7)]),
        most_popular: None,
    }
}

#[test]
fn smoke_store_get_record_top_and_disable_gate() {
    let dir = unique_test_dir("smoke");
    let mut svc = SearchService::new(&dir);

    // Enabled by default.
    assert!(svc.enabled());

    // store -> cached round-trips.
    svc.store("Pink Floyd", &sample_results());
    let got = svc.cached("Pink Floyd").expect("cached entry");
    assert_eq!(
        got.albums.items.iter().map(|a| a.id.clone()).collect::<Vec<_>>(),
        vec!["1"]
    );
    assert_eq!(got.tracks.items.iter().map(|t| t.id).collect::<Vec<_>>(), vec![10]);
    assert_eq!(got.artists.items.iter().map(|a| a.id).collect::<Vec<_>>(), vec![100]);

    // record -> top_for_query returns it.
    svc.record_interaction("Pink Floyd", "artist", "100", InteractionAction::Favorite);
    assert_eq!(
        svc.top_for_query("Pink Floyd"),
        Some(("artist".to_string(), "100".to_string()))
    );

    // Disable: reads return None, writes no-op.
    svc.set_enabled(false);
    assert!(!svc.enabled());
    assert!(svc.cached("Pink Floyd").is_none());
    assert!(svc.top_for_query("Pink Floyd").is_none());

    // store/record are no-ops while disabled (no new data observed once re-enabled).
    svc.store("New Query", &sample_results());
    svc.record_interaction("Pink Floyd", "album", "1", InteractionAction::Play);

    // Re-enable: the data written while disabled was never stored.
    svc.set_enabled(true);
    assert!(svc.cached("New Query").is_none());
    // The pre-disable artist interaction survives; the disabled-period album bump did not.
    assert_eq!(
        svc.top_for_query("Pink Floyd"),
        Some(("artist".to_string(), "100".to_string()))
    );

    // rank_within is a no-op while disabled (order preserved).
    svc.set_enabled(false);
    let mut items = vec!["x", "y", "z"];
    svc.rank_within("Pink Floyd", "artist", &mut items, |s| s.to_string());
    assert_eq!(items, vec!["x", "y", "z"]);

    let _ = std::fs::remove_dir_all(dir);
}
