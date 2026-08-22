use std::sync::RwLock;

use super::state::ThrottleInner;
use super::ThrottleState;

#[test]
fn fresh_state_returns_default_cap() {
    let s = ThrottleState {
        inner: RwLock::new(ThrottleInner::default()),
    };
    assert_eq!(s.current_prefetch_cap(2.5, 5), 5);
}

#[test]
fn panic_mode_zeros_cap() {
    let s = ThrottleState {
        inner: RwLock::new(ThrottleInner::default()),
    };
    s.record_underrun();
    assert_eq!(s.current_prefetch_cap(2.5, 5), 0);
}

#[test]
fn surviving_ratio_zeros_prefetch() {
    let s = ThrottleState {
        inner: RwLock::new(ThrottleInner::default()),
    };
    // bw = 3.0, playback = 2.5 → ratio = 1.2 (< 1.5)
    s.record_segment_bandwidth(3.0);
    assert_eq!(s.current_prefetch_cap(2.5, 5), 0);
}

#[test]
fn cautious_ratio_allows_one() {
    let s = ThrottleState {
        inner: RwLock::new(ThrottleInner::default()),
    };
    // bw = 5.0, playback = 2.5 → ratio = 2.0 (between 1.5 and 2.5)
    s.record_segment_bandwidth(5.0);
    assert_eq!(s.current_prefetch_cap(2.5, 5), 1);
}

#[test]
fn relaxed_ratio_allows_two() {
    let s = ThrottleState {
        inner: RwLock::new(ThrottleInner::default()),
    };
    // bw = 8.0, playback = 2.5 → ratio = 3.2 (between 2.5 and 4.0)
    s.record_segment_bandwidth(8.0);
    assert_eq!(s.current_prefetch_cap(2.5, 5), 2);
}

#[test]
fn abundant_bandwidth_unlocks_default() {
    let s = ThrottleState {
        inner: RwLock::new(ThrottleInner::default()),
    };
    // bw = 20.0, playback = 2.5 → ratio = 8.0 (well above 4.0)
    s.record_segment_bandwidth(20.0);
    assert_eq!(s.current_prefetch_cap(2.5, 5), 5);
}

#[test]
fn cap_never_exceeds_default() {
    let s = ThrottleState {
        inner: RwLock::new(ThrottleInner::default()),
    };
    s.record_segment_bandwidth(20.0);
    // Memory profile says 1 — never raise above.
    assert_eq!(s.current_prefetch_cap(2.5, 1), 1);
}

#[test]
fn ema_smooths_spikes() {
    let s = ThrottleState {
        inner: RwLock::new(ThrottleInner::default()),
    };
    s.record_segment_bandwidth(10.0);
    s.record_segment_bandwidth(1.0);
    // EMA: 10 * 0.6 + 1 * 0.4 = 6.4
    let bw = s.current_bandwidth_mbps().unwrap();
    assert!((bw - 6.4).abs() < 0.01);
}
