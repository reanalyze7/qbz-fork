use qbz_models::DiscoverAudioInfo;

use super::map::{classify_release_type, quality_detail, quality_tier};

fn audio(bit_depth: Option<u32>) -> DiscoverAudioInfo {
    DiscoverAudioInfo {
        maximum_bit_depth: bit_depth,
        maximum_sampling_rate: Some(96.0),
        maximum_channel_count: Some(2),
    }
}

#[test]
fn quality_tier_hires_for_24_bit() {
    assert_eq!(quality_tier(Some(&audio(Some(24)))), "hires");
}

#[test]
fn classify_release_type_track_count_heuristic() {
    assert_eq!(classify_release_type(Some(1)), "Single");
    assert_eq!(classify_release_type(Some(3)), "Single");
    assert_eq!(classify_release_type(Some(4)), "EP");
    assert_eq!(classify_release_type(Some(6)), "EP");
    assert_eq!(classify_release_type(Some(12)), "Album");
    assert_eq!(classify_release_type(None), "Album");
}

#[test]
fn quality_detail_bare_without_tier_prefix() {
    // The list-row QualityBadgeFull supplies the tier label itself,
    // so the detail line is just "<depth>-bit / <rate> kHz".
    assert_eq!(quality_detail(Some(&audio(Some(24)))), "24-bit / 96 kHz");
    assert_eq!(quality_detail(None), "");
}

#[test]
fn quality_tier_cd_for_16_bit() {
    assert_eq!(quality_tier(Some(&audio(Some(16)))), "cd");
}

#[test]
fn quality_tier_empty_without_audio_info() {
    assert_eq!(quality_tier(None), "");
}
