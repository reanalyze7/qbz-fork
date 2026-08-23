use qbz_app::settings::playback::PlaybackPreferences;
use qbz_audio::settings::AudioSettings;

use crate::tui::strings as s;

use super::fields::{row_state, PField, StagedPlayback};
use super::labels::{autoplay_label, retry_label};
use super::model::PlaybackState;

fn base() -> StagedPlayback {
    PlaybackState::new(
        "hires_plus",
        true,
        &AudioSettings::default(),
        &PlaybackPreferences::default(),
    )
    .staged
}

#[test]
fn ask_renders_note_and_is_never_written() {
    let mut st = PlaybackState::new("hires_plus", true, &AudioSettings::default(), &PlaybackPreferences::default());
    // AudioSettings::default() seeds fallback_behavior = "ask".
    assert_eq!(st.staged.fallback_behavior, "ask");
    assert_eq!(retry_label("ask"), s::RETRY_ASK);
    // A save with the value still `ask` writes nothing for that key.
    assert!(st.save_keys().iter().all(|(k, _)| k != "audio.quality_fallback_behavior"));
    // Picking a concrete value makes it writable.
    st.staged.fallback_behavior = "always_skip".to_string();
    assert!(st
        .save_keys()
        .iter()
        .any(|(k, v)| k == "audio.quality_fallback_behavior" && v == "always_skip"));
}

#[test]
fn max_rate_hidden_unless_limit_on() {
    let mut p = base();
    p.limit_to_device = false;
    assert!(!row_state(PField::MaxRate, &p).0);
    p.limit_to_device = true;
    assert!(row_state(PField::MaxRate, &p).0);
}

#[test]
fn gapless_disabled_while_streaming_only_on() {
    let mut p = base();
    p.streaming_only = true;
    let (shown, enabled, reason) = row_state(PField::Gapless, &p);
    assert!(shown && !enabled);
    assert_eq!(reason, Some(s::R_STREAMING_ONLY_ON));
}

#[test]
fn resume_enabled_only_under_restore_session() {
    let mut p = base();
    p.restore_session = false;
    assert!(!row_state(PField::Resume, &p).1);
    p.restore_session = true;
    assert!(row_state(PField::Resume, &p).1);
}

#[test]
fn infinite_autoplay_renders_readonly_label() {
    assert_eq!(autoplay_label("infinite"), s::AUTOPLAY_INFINITE);
}
