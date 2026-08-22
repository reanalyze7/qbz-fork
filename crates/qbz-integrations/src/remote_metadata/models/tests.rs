use super::*;

#[test]
fn test_provider_parsing() {
    assert_eq!(
        "musicbrainz".parse::<RemoteProvider>().unwrap(),
        RemoteProvider::MusicBrainz
    );
    assert_eq!(
        "discogs".parse::<RemoteProvider>().unwrap(),
        RemoteProvider::Discogs
    );
    assert_eq!(
        "MB".parse::<RemoteProvider>().unwrap(),
        RemoteProvider::MusicBrainz
    );
    assert!("unknown".parse::<RemoteProvider>().is_err());
}

#[test]
fn test_search_request_limit() {
    let req = RemoteSearchRequest {
        provider: RemoteProvider::MusicBrainz,
        query: "test".to_string(),
        catalog_id: None,
        artist: None,
        limit: None,
    };
    assert_eq!(req.limit(), 10);

    let req_custom = RemoteSearchRequest {
        limit: Some(50),
        ..req.clone()
    };
    assert_eq!(req_custom.limit(), 25); // Capped at 25
}
