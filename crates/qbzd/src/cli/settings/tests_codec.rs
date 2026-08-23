// crates/qbzd/src/cli/settings/tests_codec.rs — value parse/render unit
// tests for the codec modules.

use qbz_app::settings::playback::AutoplayMode;
use qbz_audio::AudioBackendType;

use super::codec_bool::{parse_backend, parse_bool};
use super::codec_playback::{parse_autoplay, parse_streaming_quality};
use super::codec_value::{parse_dsd_mode, parse_opt_u32, parse_quality_fallback_behavior, parse_stream_buffer_seconds};

#[test]
fn parse_bool_accepts_common_spellings_and_rejects_garbage() {
    assert_eq!(parse_bool("true"), Ok(true));
    assert_eq!(parse_bool("On"), Ok(true));
    assert_eq!(parse_bool("1"), Ok(true));
    assert_eq!(parse_bool("false"), Ok(false));
    assert_eq!(parse_bool("Off"), Ok(false));
    assert!(parse_bool("maybe").is_err());
}

#[test]
fn parse_backend_accepts_the_five_concrete_backends_only() {
    assert_eq!(parse_backend("alsa"), Ok(Some(AudioBackendType::Alsa)));
    assert_eq!(parse_backend("PipeWire"), Ok(Some(AudioBackendType::PipeWire)));
    assert_eq!(parse_backend("system"), Ok(Some(AudioBackendType::SystemDefault)));
    assert!(parse_backend("auto").is_err(), "Auto is omitted in v1 (03-setup-tui.md §3.2.1)");
    assert!(parse_backend("bogus").is_err());
}

#[test]
fn parse_quality_fallback_behavior_rejects_ask() {
    assert_eq!(parse_quality_fallback_behavior("always_fallback"), Ok("always_fallback".into()));
    assert_eq!(parse_quality_fallback_behavior("always_skip"), Ok("always_skip".into()));
    let err = parse_quality_fallback_behavior("ask").unwrap_err();
    assert!(err.contains("needs a UI"), "{err}");
}

#[test]
fn parse_streaming_quality_matches_the_four_canonical_keys() {
    for ok in ["mp3", "cd", "hires", "hires_plus"] {
        assert!(parse_streaming_quality(ok).is_ok(), "{ok}");
    }
    assert!(parse_streaming_quality("hires192").is_err(), "not a real key — see report");
}

#[test]
fn parse_autoplay_matches_the_playback_preferences_wire_values() {
    assert_eq!(parse_autoplay("continue").unwrap(), AutoplayMode::ContinueWithinSource);
    assert_eq!(parse_autoplay("track_only").unwrap(), AutoplayMode::PlayTrackOnly);
    assert_eq!(parse_autoplay("infinite").unwrap(), AutoplayMode::InfiniteRadio);
    assert!(parse_autoplay("bogus").is_err());
}

#[test]
fn parse_opt_u32_clears_on_none_or_empty() {
    assert_eq!(parse_opt_u32(""), Ok(None));
    assert_eq!(parse_opt_u32("none"), Ok(None));
    assert_eq!(parse_opt_u32("192000"), Ok(Some(192_000)));
    assert!(parse_opt_u32("loud").is_err());
}

#[test]
fn parse_dsd_mode_rejects_unknown_modes() {
    for ok in ["convert", "dop", "native"] {
        assert!(parse_dsd_mode(ok).is_ok());
    }
    assert!(parse_dsd_mode("bogus").is_err());
}

#[test]
fn parse_stream_buffer_seconds_enforces_1_to_10() {
    assert_eq!(parse_stream_buffer_seconds("2"), Ok(2));
    assert_eq!(parse_stream_buffer_seconds("10"), Ok(10));
    assert!(parse_stream_buffer_seconds("0").is_err());
    assert!(parse_stream_buffer_seconds("11").is_err());
}
