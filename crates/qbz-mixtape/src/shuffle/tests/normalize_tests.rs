use crate::shuffle::{normalize_artist, normalize_title, token_set_ratio, SIMILARITY_THRESHOLD};

// ──────── normalize_title ────────

#[test]
fn normalize_title_lowercases() {
    assert_eq!(normalize_title("Yesterday"), "yesterday");
}

#[test]
fn normalize_title_strips_parens() {
    assert_eq!(normalize_title("Yesterday (Live)"), "yesterday");
    assert_eq!(normalize_title("Song (Acoustic Version)"), "song");
}

#[test]
fn normalize_title_strips_brackets_and_braces() {
    assert_eq!(normalize_title("Track [Bonus]"), "track");
    assert_eq!(normalize_title("Tune {Demo}"), "tune");
}

#[test]
fn normalize_title_strips_dash_suffix() {
    assert_eq!(normalize_title("Song - 2003 Remaster"), "song");
    assert_eq!(normalize_title("Tune - Live at Wembley"), "tune");
}

#[test]
fn normalize_title_strips_feat() {
    assert_eq!(normalize_title("Song feat. Artist X"), "song");
    assert_eq!(normalize_title("Tune ft. X"), "tune");
    assert_eq!(normalize_title("Anthem featuring Y"), "anthem");
}

#[test]
fn normalize_title_strips_diacritics() {
    assert_eq!(normalize_title("Café"), "cafe");
    assert_eq!(normalize_title("Niño"), "nino");
    assert_eq!(normalize_title("Über"), "uber");
}

#[test]
fn normalize_title_strips_punctuation() {
    assert_eq!(normalize_title("Don't Stop!"), "dont stop");
    assert_eq!(normalize_title("¿Qué Pasa?"), "que pasa");
}

#[test]
fn normalize_title_collapses_whitespace() {
    assert_eq!(normalize_title("  Hello   World  "), "hello world");
}

#[test]
fn normalize_title_combined() {
    assert_eq!(
        normalize_title("¡Yesterday! (Live, Wembley) - 2003 Remaster feat. Friend"),
        "yesterday"
    );
}

// ──────── normalize_artist ────────

#[test]
fn normalize_artist_lowercases_and_trims() {
    assert_eq!(normalize_artist("  The Beatles  "), "the beatles");
}

#[test]
fn normalize_artist_strips_diacritics() {
    assert_eq!(normalize_artist("Mägo de Oz"), "mago de oz");
}

#[test]
fn normalize_artist_keeps_parens() {
    // "Foo (band)" must NOT collapse to "Foo" — that's title behavior, not artist.
    assert_eq!(normalize_artist("Foo (band)"), "foo (band)");
}

// ──────── token_set_ratio ────────

#[test]
fn token_set_ratio_identical_returns_one() {
    assert!((token_set_ratio("yesterday", "yesterday") - 1.0).abs() < 1e-6);
}

#[test]
fn token_set_ratio_subset_returns_one() {
    // RapidFuzz behavior: when one string's tokens are a subset of the
    // other, t1 == t2 (the smaller side), so max similarity is 1.0.
    let s = token_set_ratio("yesterday", "yesterday live wembley");
    assert!(s >= 0.95, "expected >= 0.95, got {}", s);
}

#[test]
fn token_set_ratio_disjoint_returns_low() {
    let s = token_set_ratio("yesterday", "tomorrow");
    assert!(s < 0.50, "expected < 0.50, got {}", s);
}

#[test]
fn token_set_ratio_overlap_passes_threshold() {
    // Three of four words shared after normalization.
    let s = token_set_ratio("song of the south", "song of the north");
    assert!(s >= SIMILARITY_THRESHOLD, "expected >= 0.80, got {}", s);
}

#[test]
fn token_set_ratio_unrelated_fails_threshold() {
    let s = token_set_ratio("a totally different song", "yesterday");
    assert!(s < SIMILARITY_THRESHOLD, "expected < 0.80, got {}", s);
}

#[test]
fn token_set_ratio_empty_inputs_safe() {
    // Both empty → defined as 1.0 (vacuously identical).
    assert!((token_set_ratio("", "") - 1.0).abs() < 1e-6);
    // One empty → 0.0 (no overlap).
    assert!(token_set_ratio("", "yesterday") < 1e-6);
}
