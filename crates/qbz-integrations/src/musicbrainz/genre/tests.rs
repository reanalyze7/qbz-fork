use super::*;
use crate::musicbrainz::Tag;

#[test]
fn test_normalize_genre() {
    assert_eq!(normalize_genre("alt rock"), "alternative rock");
    assert_eq!(normalize_genre("Alt Rock"), "alternative rock");
    assert_eq!(normalize_genre("grunge rock"), "grunge");
    assert_eq!(normalize_genre("prog"), "progressive rock");
    assert_eq!(normalize_genre("hip hop"), "hip hop");
    assert_eq!(normalize_genre("hip-hop"), "hip hop");
    assert_eq!(normalize_genre("unknown genre"), "unknown genre");
}

#[test]
fn test_noisy_tags_filtered() {
    let tags = vec![
        Tag {
            name: "rock".to_string(),
            count: Some(10),
        },
        Tag {
            name: "seen live".to_string(),
            count: Some(8),
        },
        Tag {
            name: "awesome".to_string(),
            count: Some(5),
        },
        Tag {
            name: "grunge".to_string(),
            count: Some(4),
        },
    ];

    let seeds = extract_affinity_seeds(&tags);
    assert_eq!(seeds.genres, vec!["rock", "grunge"]);
    assert!(seeds.tags.is_empty());
}

#[test]
fn test_empty_tags() {
    let seeds = extract_affinity_seeds(&[]);
    assert!(seeds.genres.is_empty());
    assert!(seeds.tags.is_empty());
    assert!(seeds.normalized_seeds.is_empty());
}
