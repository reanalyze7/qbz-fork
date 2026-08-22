use super::*;

// ── select_best_match ──

#[test]
fn select_best_match_skips_non_streamable() {
    let source = import_track("hey jude", "the beatles");
    let mut perfect = qobuz_track(1, "hey jude", "the beatles");
    perfect.streamable = false;
    let weaker = qobuz_track(2, "hey jude na na", "the beatles");

    let candidates = [perfect, weaker];
    let (best, _score) = select_best_match(&source, &candidates);
    assert_eq!(best.map(|t| t.id), Some(2));
}

#[test]
fn select_best_match_score_below_min_score_threshold() {
    // select_best_match does NOT gate on MIN_SCORE — the CALLER
    // (match_tracks) drops sub-threshold matches. Prove a PARTIAL match
    // (same artist, different title → ~0.3, well under 0.65) is still
    // RETURNED, leaving the rejection to the caller. NOTE: it must be a
    // partial, not pure junk — a fully disjoint candidate scores 0.0,
    // never beats the 0.0 seed (`score > best_score + 0.0001`), and comes
    // back as None, which is a different code path.
    let source = import_track("hey jude", "the beatles");
    let partial = qobuz_track(1, "let it be", "the beatles");
    let candidates = [partial];
    let (best, score) = select_best_match(&source, &candidates);
    assert!(best.is_some(), "a partial match must still be returned");
    assert!(score > 0.0 && score < MIN_SCORE, "got {}", score);
}

#[test]
fn select_best_match_hi_res_tiebreak_within_001() {
    let source = import_track("hey jude", "the beatles");
    let mut cd = qobuz_track(1, "hey jude", "the beatles");
    cd.maximum_bit_depth = Some(16);
    cd.maximum_sampling_rate = Some(44.1);
    let mut hires = qobuz_track(2, "hey jude", "the beatles");
    hires.maximum_bit_depth = Some(24);
    hires.maximum_sampling_rate = Some(192.0);

    // Equal scores → quality_score decides, regardless of order.
    let ordered = [cd.clone(), hires.clone()];
    let (best, _) = select_best_match(&source, &ordered);
    assert_eq!(best.map(|t| t.id), Some(2));
    let reversed = [hires, cd];
    let (best, _) = select_best_match(&source, &reversed);
    assert_eq!(best.map(|t| t.id), Some(2));
}

#[test]
fn select_best_match_empty_candidates() {
    let source = import_track("hey jude", "the beatles");
    let (best, score) = select_best_match(&source, &[]);
    assert!(best.is_none());
    assert_eq!(score, 0.0);
}

// ── quality_score ──

#[test]
fn quality_score_weighs_bit_depth_over_sample_rate() {
    let mut a = qobuz_track(1, "x", "y");
    a.maximum_bit_depth = Some(24);
    a.maximum_sampling_rate = Some(44.1);
    let mut b = qobuz_track(2, "x", "y");
    b.maximum_bit_depth = Some(16);
    b.maximum_sampling_rate = Some(192.0);
    assert!(quality_score(&a) > quality_score(&b));

    let none = qobuz_track(3, "x", "y");
    assert_eq!(quality_score(&none), 0.0);
}
