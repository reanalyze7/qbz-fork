use super::fixtures::sample_track;
use crate::api::queue::shared::{paginate, parse_offset_limit};

// ---------------------------- pagination ----------------------------

#[test]
fn paginate_slices_offset_and_limit() {
    let items: Vec<_> = (0..5).map(sample_track).collect();
    let page = paginate(&items, 1, 2);
    assert_eq!(page.iter().map(|t| t.id).collect::<Vec<_>>(), vec![1, 2]);
}

#[test]
fn paginate_offset_past_end_is_empty() {
    let items: Vec<_> = (0..3).map(sample_track).collect();
    assert!(paginate(&items, 10, 5).is_empty());
}

#[test]
fn paginate_limit_larger_than_remaining_returns_the_rest() {
    let items: Vec<_> = (0..3).map(sample_track).collect();
    let page = paginate(&items, 1, 100);
    assert_eq!(page.iter().map(|t| t.id).collect::<Vec<_>>(), vec![1, 2]);
}

#[test]
fn parse_offset_limit_defaults_when_absent() {
    assert_eq!(parse_offset_limit(""), (0, 100));
    assert_eq!(parse_offset_limit("foo=bar"), (0, 100));
}

#[test]
fn parse_offset_limit_reads_both_params() {
    assert_eq!(parse_offset_limit("offset=5&limit=10"), (5, 10));
    assert_eq!(parse_offset_limit("limit=10&offset=5"), (5, 10));
}

#[test]
fn parse_offset_limit_ignores_malformed_values() {
    assert_eq!(parse_offset_limit("offset=nope&limit=10"), (0, 10));
}

#[test]
fn repeat_str_matches_contract_lowercase() {
    use crate::api::queue::mapping::repeat_str;
    assert_eq!(repeat_str(qbz_models::RepeatMode::Off), "off");
    assert_eq!(repeat_str(qbz_models::RepeatMode::All), "all");
    assert_eq!(repeat_str(qbz_models::RepeatMode::One), "one");
}
