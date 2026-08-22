use crate::weights::RelationshipWeights;

// Integration tests would require mocking the clients; here we cover the
// weight ordering the builder relies on.
#[test]
fn test_weights_applied() {
    let weights = RelationshipWeights::default();

    assert!(weights.member_of_band > weights.collaboration);
    assert!(weights.collaboration > weights.shared_tag);
}
