use super::map::{format_duration, mmss, tier};

#[test]
fn mmss_pads_seconds() {
    assert_eq!(mmss(5), "0:05");
    assert_eq!(mmss(65), "1:05");
    assert_eq!(mmss(225), "3:45");
}

#[test]
fn duration_drops_zero_hours() {
    assert_eq!(format_duration(2700), "45m");
    assert_eq!(format_duration(3720), "1h 2m");
}

#[test]
fn tier_classifies_bit_depth() {
    assert_eq!(tier(Some(24)), "hires");
    assert_eq!(tier(Some(16)), "cd");
    assert_eq!(tier(None), "");
}
