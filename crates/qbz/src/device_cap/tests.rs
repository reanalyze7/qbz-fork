use qbz_models::Quality;

use super::state::tier_for_max_rate_hz;
use super::summary::rate_khz_label;

#[test]
fn tier_mapping_matches_spec_c3_table() {
    // > 96 kHz → Hi-Res+ (no effective cap).
    assert_eq!(tier_for_max_rate_hz(192_000), Quality::UltraHiRes);
    assert_eq!(tier_for_max_rate_hz(176_400), Quality::UltraHiRes);
    // 96 / 88.2 kHz → Hi-Res.
    assert_eq!(tier_for_max_rate_hz(96_000), Quality::HiRes);
    assert_eq!(tier_for_max_rate_hz(88_200), Quality::HiRes);
    // ≤ 48 kHz → CD 16/44.1 (bit depth lost too — no 48 kHz tier).
    assert_eq!(tier_for_max_rate_hz(48_000), Quality::Lossless);
    assert_eq!(tier_for_max_rate_hz(44_100), Quality::Lossless);
}

#[test]
fn rate_label_formats_whole_and_fractional_khz() {
    assert_eq!(rate_khz_label(192_000), "192 kHz");
    assert_eq!(rate_khz_label(44_100), "44.1 kHz");
    assert_eq!(rate_khz_label(176_400), "176.4 kHz");
}
