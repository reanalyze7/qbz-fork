use super::*;

#[test]
fn test_extract_artist_mbids() {
    let tracks = vec![
        (1, Some("mbid-1".to_string())),
        (2, Some("mbid-2".to_string())),
        (3, Some("mbid-1".to_string())), // Duplicate
        (4, None),                       // No MBID
        (5, Some("".to_string())),       // Empty MBID
        (6, Some("mbid-3".to_string())),
    ];

    let mbids = extract_artist_mbids(&tracks);

    assert_eq!(mbids.len(), 3);
    assert!(mbids.contains(&"mbid-1".to_string()));
    assert!(mbids.contains(&"mbid-2".to_string()));
    assert!(mbids.contains(&"mbid-3".to_string()));
}

#[test]
fn test_suggestion_config_default() {
    let config = SuggestionConfig::default();

    assert_eq!(config.max_artists, 30);
    assert_eq!(config.tracks_per_artist, 6);
    assert_eq!(config.max_pool_size, 150);
    assert_eq!(config.vector_max_age_days, 7);
    assert!(config.min_similarity > 0.0);
}
