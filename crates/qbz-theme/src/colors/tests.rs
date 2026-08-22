use super::*;
use crate::color::Rgba;

#[test]
fn alpha_byte_rounds() {
    assert_eq!(alpha_byte(8), 0x14); // 8% -> 20
    assert_eq!(alpha_byte(10), 0x1a); // 10% -> 26
    assert_eq!(alpha_byte(12), 0x1f); // 12% -> 31
    assert_eq!(alpha_byte(18), 0x2e); // 18% -> 46
    assert_eq!(alpha_byte(55), 0x8c); // 55% -> 140
    assert_eq!(alpha_byte(65), 0xa6); // 65% -> 166
    assert_eq!(alpha_byte(70), 0xb3); // 70% -> 179
    assert_eq!(alpha_byte(75), 0xbf); // 75% -> 191
}

#[test]
fn ramp_polarity() {
    let dark = alpha_ramp(false);
    let light = alpha_ramp(true);
    assert_eq!(dark[alpha_index(8).unwrap()], Rgba::rgba(255, 255, 255, 0x14));
    assert_eq!(light[alpha_index(8).unwrap()], Rgba::rgba(0, 0, 0, 0x14));
}

#[test]
fn alpha_count_is_24() {
    assert_eq!(ALPHA_COUNT, 24);
    assert_eq!(ALPHA_PERCENTS.len(), 24);
}
