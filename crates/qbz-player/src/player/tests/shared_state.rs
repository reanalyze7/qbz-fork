use super::*;

#[test]
fn current_position_ms_is_a_pure_anchor_derivation() {
    use std::sync::atomic::Ordering;

    let state = SharedState::new();
    state.duration.store(300, Ordering::SeqCst);

    // Paused: coarse stored position scaled to ms (current_position parity).
    state.position.store(12, Ordering::SeqCst);
    assert_eq!(state.current_position_ms(), 12_000);

    // Playing without an anchor yet: same coarse fallback.
    state.is_playing.store(true, Ordering::SeqCst);
    assert_eq!(state.current_position_ms(), 12_000);

    // Playing with anchors: position_at_start*1000 + wall-clock elapsed ms.
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    state
        .playback_start_millis
        .store(now_ms - 1_500, Ordering::SeqCst);
    state.position_at_start.store(10, Ordering::SeqCst);
    let pos = state.current_position_ms();
    assert!(
        (11_450..=11_900).contains(&pos),
        "expected ~11500ms, got {pos}"
    );

    // Clamped to duration*1000 (same rule current_position applies).
    state
        .playback_start_millis
        .store(now_ms - 10_000, Ordering::SeqCst);
    state.position_at_start.store(299, Ordering::SeqCst);
    assert_eq!(state.current_position_ms(), 300_000);
}

#[test]
fn stream_quality_normalizes_units() {
    use qbz_models::StreamQualityInfo;
    // kHz input stays kHz.
    let khz = StreamQualityInfo::from_raw(7, Some(96.0), Some(24));
    assert_eq!(khz.sampling_rate_khz, Some(96.0));
    // Hz input is converted to kHz.
    let hz = StreamQualityInfo::from_raw(27, Some(192000.0), Some(24));
    assert_eq!(hz.sampling_rate_khz, Some(192.0));
    // Zero / unknown -> None.
    let zero = StreamQualityInfo::from_raw(6, Some(0.0), Some(16));
    assert_eq!(zero.sampling_rate_khz, None);
    // Tier label from format id.
    assert_eq!(khz.tier_label(), "FLAC 24-bit/≤96kHz");
    assert_eq!(hz.tier_label(), "FLAC 24-bit/>96kHz");
}
