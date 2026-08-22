use super::text::token_overlap;
use super::*;

#[test]
fn normalize_strips_brackets_and_stop_words() {
    assert_eq!(normalize("Song Title (Remastered 2011)"), "song title");
    assert_eq!(normalize("Hey Jude - Remastered"), "hey jude");
}

#[test]
fn similarity_exact_and_substring() {
    assert_eq!(similarity("Hey Jude (Remastered)", "hey jude"), 1.0);
    assert_eq!(similarity("hey jude", "hey jude na na"), 0.85);
    assert_eq!(similarity("", "anything"), 0.0);
}

#[test]
fn token_overlap_uses_longer_side() {
    assert_eq!(token_overlap("a b", "a b c d"), 0.5);
    assert_eq!(token_overlap("x", "y"), 0.0);
}

#[test]
fn normalize_is_stable_for_cache_keys() {
    assert_eq!(normalize("  The Beatles  "), "the beatles");
    assert_eq!(normalize("AC/DC"), "ac dc");
}
