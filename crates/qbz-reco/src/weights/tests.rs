use super::*;

#[test]
fn test_default_weights() {
    let weights = RelationshipWeights::default();

    assert_eq!(weights.member_of_band, 1.0);
    assert!(weights.qobuz_similar > weights.shared_tag);
    assert!(weights.collaboration > weights.engineer);
}

#[test]
fn test_mb_relation_weight() {
    let weights = RelationshipWeights::default();

    assert_eq!(weights.weight_for_mb_relation("member of band"), 1.0);
    assert_eq!(weights.weight_for_mb_relation("collaboration"), 0.8);
    assert_eq!(weights.weight_for_mb_relation("producer"), 0.5);

    // Unknown type gets default
    assert_eq!(weights.weight_for_mb_relation("unknown_type"), 0.2);
}

#[test]
fn test_source_weight() {
    let weights = RelationshipWeights::default();

    assert_eq!(weights.weight_for_source("qobuz_similar"), 0.7);
    assert_eq!(weights.weight_for_source("mb:member of band"), 1.0);
    assert_eq!(weights.weight_for_source("mb:collaboration"), 0.8);
}
