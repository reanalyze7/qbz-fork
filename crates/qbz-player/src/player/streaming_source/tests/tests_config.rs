//! Buffer-size ladder tests for `StreamingConfig::from_speed_mbps*` and the
//! process-wide cap (issue #331).

use crate::player::streaming_source::cap::raw_initial_buffer_for_speed;
use crate::player::streaming_source::StreamingConfig;

#[test]
fn raw_initial_buffer_for_speed_follows_documented_ladder() {
    // Each band of the documented speed ladder produces its own size.
    assert_eq!(raw_initial_buffer_for_speed(20.0), 256 * 1024);
    assert_eq!(raw_initial_buffer_for_speed(10.0), 256 * 1024);
    assert_eq!(raw_initial_buffer_for_speed(7.0), 384 * 1024);
    assert_eq!(raw_initial_buffer_for_speed(5.0), 384 * 1024);
    assert_eq!(raw_initial_buffer_for_speed(3.0), 512 * 1024);
    assert_eq!(raw_initial_buffer_for_speed(2.0), 512 * 1024);
    assert_eq!(raw_initial_buffer_for_speed(1.5), 1024 * 1024);
    assert_eq!(raw_initial_buffer_for_speed(1.0), 1024 * 1024);
    assert_eq!(raw_initial_buffer_for_speed(0.5), 2 * 1024 * 1024);
    assert_eq!(raw_initial_buffer_for_speed(0.0), 2 * 1024 * 1024);
}

#[test]
fn from_speed_mbps_with_cap_passes_through_when_under_cap() {
    // Cap above the raw value: result equals the raw ladder.
    let cfg = StreamingConfig::from_speed_mbps_with_cap(0.0, 4 * 1024 * 1024);
    assert_eq!(cfg.initial_buffer_bytes, 2 * 1024 * 1024);
}

#[test]
fn from_speed_mbps_with_cap_clamps_slow_connection_to_low_memory_cap() {
    // The case from issue #331: Pi 3B, slow connection because of swap
    // thrash, would otherwise inflate the buffer to 2 MB. With the
    // LowMemory profile's 256KB cap applied, we stay at 256KB.
    let cfg = StreamingConfig::from_speed_mbps_with_cap(0.0, 256 * 1024);
    assert_eq!(cfg.initial_buffer_bytes, 256 * 1024);

    let cfg = StreamingConfig::from_speed_mbps_with_cap(1.5, 256 * 1024);
    assert_eq!(cfg.initial_buffer_bytes, 256 * 1024);
}

#[test]
fn from_speed_mbps_with_cap_no_op_for_normal_profile() {
    // Normal profile cap is 2 MB — equal to the slowest raw band, so
    // any raw value passes through unchanged.
    let cap = 2 * 1024 * 1024;
    for speed in [0.0, 0.5, 1.0, 2.0, 5.0, 10.0, 20.0] {
        let cfg = StreamingConfig::from_speed_mbps_with_cap(speed, cap);
        assert_eq!(
            cfg.initial_buffer_bytes,
            raw_initial_buffer_for_speed(speed),
            "cap should not bind for speed={}",
            speed
        );
    }
}

#[test]
fn from_speed_mbps_with_cap_max_buffer_unchanged() {
    // Whatever the cap, the secondary max_buffer_bytes stays at its
    // module default; we are only clamping the initial fill target.
    let cfg = StreamingConfig::from_speed_mbps_with_cap(0.5, 64 * 1024);
    assert_eq!(cfg.max_buffer_bytes, 100 * 1024 * 1024);
}
