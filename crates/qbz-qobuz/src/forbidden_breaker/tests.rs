use super::*;

#[test]
fn stays_closed_below_threshold() {
    let b = ForbiddenBreaker::new();
    assert!(b.record_forbidden().is_none());
    assert!(b.record_forbidden().is_none());
    assert!(b.blocked_for().is_none(), "2 < threshold, still closed");
}

#[test]
fn opens_at_threshold_and_blocks() {
    let b = ForbiddenBreaker::new();
    b.record_forbidden();
    b.record_forbidden();
    let opened = b.record_forbidden();
    assert_eq!(opened, Some(BASE_COOLDOWN), "3rd 403 opens with base cooldown");
    let remaining = b.blocked_for().expect("breaker is open");
    assert!(remaining <= BASE_COOLDOWN && remaining > Duration::ZERO);
}

#[test]
fn success_resets_everything() {
    let b = ForbiddenBreaker::new();
    b.record_forbidden();
    b.record_forbidden();
    b.record_success();
    // Counter cleared: it takes another full threshold to open again.
    assert!(b.record_forbidden().is_none());
    assert!(b.record_forbidden().is_none());
    assert!(b.blocked_for().is_none());
}

#[test]
fn cooldown_grows_exponentially_and_caps() {
    let b = ForbiddenBreaker::new();
    // First open: BASE.
    b.record_forbidden();
    b.record_forbidden();
    assert_eq!(b.record_forbidden(), Some(BASE_COOLDOWN));
    // Consecutive is not reset on open, so a single further 403 re-opens
    // with the doubled cooldown.
    assert_eq!(b.record_forbidden(), Some(BASE_COOLDOWN * 2));
    assert_eq!(b.record_forbidden(), Some((BASE_COOLDOWN * 4).min(MAX_COOLDOWN)));
    // Keep re-opening; cooldown must never exceed the cap.
    for _ in 0..8 {
        let c = b.record_forbidden().expect("re-opens each time past threshold");
        assert!(c <= MAX_COOLDOWN);
    }
    assert_eq!(b.record_forbidden(), Some(MAX_COOLDOWN), "cooldown pinned at cap");
}
