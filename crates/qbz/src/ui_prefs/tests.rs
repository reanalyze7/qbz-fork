use super::defaults::default_theme;
use super::model::UiPrefs;
use super::quality::{streaming_quality_for_key, streaming_quality_index, STREAMING_QUALITIES};
use qbz_models::Quality;

#[test]
fn default_is_hires_plus() {
    assert_eq!(UiPrefs::default().streaming_quality, "hires_plus");
    assert_eq!(STREAMING_QUALITIES.len(), 4);
    assert_eq!(STREAMING_QUALITIES[3].key, "hires_plus");
}

#[test]
fn unknown_key_resolves_to_default_index() {
    // Default is hires_plus, which is index 3.
    assert_eq!(streaming_quality_index("bogus"), 3);
    assert_eq!(streaming_quality_index("mp3"), 0);
    assert_eq!(streaming_quality_index("cd"), 1);
}

#[test]
fn quality_key_maps_to_qobuz_format_id() {
    assert_eq!(streaming_quality_for_key("mp3"), Quality::Mp3);
    assert_eq!(streaming_quality_for_key("cd"), Quality::Lossless);
    assert_eq!(streaming_quality_for_key("hires"), Quality::HiRes);
    assert_eq!(streaming_quality_for_key("hires_plus"), Quality::UltraHiRes);
    // Unknown/unset keys fall back to the default tier.
    assert_eq!(streaming_quality_for_key("bogus"), Quality::UltraHiRes);
    assert_eq!(streaming_quality_for_key(""), Quality::UltraHiRes);
}

#[test]
fn legacy_json_without_field_deserializes() {
    let prefs: UiPrefs = serde_json::from_str("{}").expect("empty object deserializes");
    assert_eq!(prefs.streaming_quality, "hires_plus");
    assert!(prefs.album_header_gradient);
    // A profile that predates the theme field falls back to OLED.
    assert_eq!(prefs.theme, "oled");
}

#[test]
fn default_theme_is_oled() {
    assert_eq!(UiPrefs::default().theme, "oled");
    assert_eq!(default_theme(), "oled");
}
