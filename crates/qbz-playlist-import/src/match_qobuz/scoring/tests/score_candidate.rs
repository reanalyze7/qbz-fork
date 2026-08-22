use super::*;

#[test]
fn score_candidate_isrc_short_circuit_is_case_insensitive() {
    let mut source = import_track("Completely Different", "Nobody");
    source.isrc = Some("uskO11600123".to_string());
    let mut candidate = qobuz_track(1, "Other Title", "Other Artist");
    candidate.isrc = Some("USKO11600123".to_string());
    assert_eq!(score_candidate(&source, &candidate), 1.0);
}

#[test]
fn score_candidate_weights_title_artist_album() {
    let mut source = import_track("hey jude", "the beatles");
    source.album = Some("past masters".to_string());
    let mut candidate = qobuz_track(1, "hey jude", "the beatles");
    candidate.album = Some(album_summary("past masters"));
    // duration_ms is None on the source → no duration bonus, so the score
    // is exactly the 0.6/0.3/0.1 weighted sum (all components 1.0 here).
    let score = score_candidate(&source, &candidate);
    assert!((score - 1.0).abs() < 1e-6, "got {}", score);
}

#[test]
fn score_candidate_duration_bonus_tiers() {
    let mut source = import_track("hey jude", "the beatles");
    let mut candidate = qobuz_track(1, "hey jude", "the beatles");
    candidate.duration = 200; // 200_000 ms

    // No source duration → no bonus (source-duration-only quirk).
    let base = score_candidate(&source, &candidate);
    assert!((base - 0.9).abs() < 1e-6, "got {}", base);

    // Within 3s → +0.05
    source.duration_ms = Some(202_000);
    let close = score_candidate(&source, &candidate);
    assert!((close - 0.95).abs() < 1e-6, "got {}", close);

    // Within 5s → +0.02
    source.duration_ms = Some(204_500);
    let near = score_candidate(&source, &candidate);
    assert!((near - 0.92).abs() < 1e-6, "got {}", near);

    // Beyond 5s → no bonus
    source.duration_ms = Some(210_000);
    let far = score_candidate(&source, &candidate);
    assert!((far - 0.9).abs() < 1e-6, "got {}", far);
}
