use super::{LocalFavItem, LocalFavoritesService};

fn item(kind: &str, id: &str, title: &str, artist: &str, source: &str) -> LocalFavItem {
    LocalFavItem {
        kind: kind.to_string(),
        id: id.to_string(),
        title: title.to_string(),
        subtitle: String::new(),
        artwork_url: String::new(),
        artist: artist.to_string(),
        source: source.to_string(),
        favorited_at: 0,
    }
}

#[test]
fn lifecycle() {
    let s = LocalFavoritesService::new_in_memory().expect("svc");
    assert!(!s.is_favorite("album", "al:abc"));
    assert_eq!(s.count(), 0);

    s.favorite(&item("album", "al:abc", "A", "Artist X", "local"))
        .unwrap();
    assert!(s.is_favorite("album", "al:abc"));
    assert!(!s.is_favorite("track", "al:abc"));

    s.favorite(&item("track", "/music/x.flac", "T", "Artist X", "local"))
        .unwrap();
    assert_eq!(s.count(), 2);
    let by_artist = s.count_by_artist().unwrap();
    assert_eq!(by_artist[0], ("Artist X".to_string(), 2));

    let all = s.list().unwrap();
    assert_eq!(all.len(), 2);

    s.unfavorite("album", "al:abc").unwrap();
    assert!(!s.is_favorite("album", "al:abc"));
    assert_eq!(s.count(), 1);
    s.unfavorite("album", "nope").unwrap();
}

#[test]
fn source_check_rejects_offline() {
    let s = LocalFavoritesService::new_in_memory().expect("svc");
    assert!(
        s.favorite(&item("album", "x", "X", "A", "qobuz_download"))
            .is_err(),
        "the source CHECK refuses qobuz-offline rows"
    );
}
