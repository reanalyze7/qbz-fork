use super::*;

// ── normalize / remove_bracketed / token_overlap ──

#[test]
fn normalize_strips_brackets_and_stop_words() {
    assert_eq!(normalize("Song Title (Remastered 2011)"), "song title");
    assert_eq!(normalize("Song Title [Deluxe Edition]"), "song title");
    assert_eq!(normalize("Hey Jude - Remastered"), "hey jude");
}

#[test]
fn normalize_handles_nested_brackets() {
    assert_eq!(normalize("Title (Live [2020] Version)"), "title");
}

#[test]
fn normalize_degrades_non_ascii_to_spaces() {
    // Accented/CJK chars are not ASCII-alphanumeric → replaced by spaces.
    // Locks the (lossy) accented-title behavior of the Tauri original.
    assert_eq!(normalize("Café Tacvba"), "caf tacvba");
    assert_eq!(normalize("너의 의미"), "");
}

#[test]
fn normalize_lowercases_and_collapses_punctuation() {
    assert_eq!(normalize("AC/DC - T.N.T."), "ac dc t n t");
}

#[test]
fn remove_bracketed_basic_and_nested() {
    assert_eq!(remove_bracketed("a (b) c"), "a  c");
    assert_eq!(remove_bracketed("a (b [c] d) e"), "a  e");
    assert_eq!(remove_bracketed("no brackets"), "no brackets");
    // Unbalanced closers are ignored at depth 0
    assert_eq!(remove_bracketed(") leading"), " leading");
}

#[test]
fn token_overlap_ratio_uses_longer_side() {
    assert_eq!(token_overlap("a b", "a b"), 1.0);
    assert_eq!(token_overlap("a b", "a b c d"), 0.5);
    assert_eq!(token_overlap("x", "y"), 0.0);
    assert_eq!(token_overlap("", "a"), 0.0);
}

// ── similarity ──

#[test]
fn similarity_exact_after_normalization_is_one() {
    assert_eq!(similarity("Hey Jude (Remastered)", "hey jude"), 1.0);
}

#[test]
fn similarity_substring_is_085() {
    assert_eq!(similarity("hey jude", "hey jude na na"), 0.85);
}

#[test]
fn similarity_falls_back_to_token_overlap() {
    // "hey there" vs "jude there": 1 shared token / max(2, 2) = 0.5
    assert_eq!(similarity("hey there", "jude there"), 0.5);
}

#[test]
fn similarity_empty_is_zero() {
    assert_eq!(similarity("", "anything"), 0.0);
    assert_eq!(similarity("anything", ""), 0.0);
}
