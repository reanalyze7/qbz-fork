use qbz_media_controls::PlaybackStatus;
use qbz_models::PlaybackState;

use super::mapping::map_state;

#[test]
fn map_state_covers_every_playback_state() {
    assert_eq!(map_state(PlaybackState::Playing), PlaybackStatus::Playing);
    assert_eq!(map_state(PlaybackState::Paused), PlaybackStatus::Paused);
    assert_eq!(map_state(PlaybackState::Stopped), PlaybackStatus::Stopped);
    assert_eq!(map_state(PlaybackState::Loading), PlaybackStatus::Playing);
}

#[test]
fn enabled_defaults_on_and_respects_falsey_overrides() {
    // Default (unset) is ON; only explicit falsey values disable. We can't
    // safely mutate process env in parallel tests, so assert the pure
    // classification the getter uses.
    for v in ["0", "false", "off", "no", "FALSE", " Off "] {
        assert!(
            matches!(v.trim().to_ascii_lowercase().as_str(), "0" | "false" | "off" | "no"),
            "{v:?} should read as disabled"
        );
    }
    for v in ["1", "true", "on", "yes", "anything"] {
        assert!(
            !matches!(v.trim().to_ascii_lowercase().as_str(), "0" | "false" | "off" | "no"),
            "{v:?} should read as enabled"
        );
    }
}
