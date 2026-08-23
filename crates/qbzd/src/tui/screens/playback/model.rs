use qbz_app::settings::playback::PlaybackPreferences;
use qbz_audio::settings::AudioSettings;

use crate::tui::strings as s;
use crate::tui::widgets::SelectPopup;

use super::fields::StagedPlayback;
use super::labels::autoplay_value;

// Note: adding a new `Editor` variant means touching this enum plus its two
// exhaustive matches in `input.rs` (`handle_editor_key`) and `render.rs`
// (`draw`'s editor-popup match).
pub(super) enum Editor {
    Quality(SelectPopup),
    MaxRate(SelectPopup),
    Retry(SelectPopup),
}

pub struct PlaybackState {
    pub(super) baseline: StagedPlayback,
    pub(super) staged: StagedPlayback,
    pub(super) focus: usize,
    pub(super) editor: Option<Editor>,
}

impl PlaybackState {
    pub fn new(quality: &str, mpris: bool, audio: &AudioSettings, prefs: &PlaybackPreferences) -> Self {
        let staged = StagedPlayback {
            quality: quality.to_string(),
            limit_to_device: audio.limit_quality_to_device,
            max_sample_rate: audio.device_max_sample_rate,
            allow_fallback: audio.allow_quality_fallback,
            fallback_behavior: audio.quality_fallback_behavior.clone(),
            autoplay: autoplay_value(prefs.autoplay_mode).to_string(),
            gapless: audio.gapless_enabled,
            restore_session: prefs.persist_session,
            resume_position: prefs.resume_playback_position,
            mpris,
            streaming_only: audio.streaming_only,
        };
        Self {
            baseline: staged.clone(),
            staged,
            focus: 0,
            editor: None,
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.staged != self.baseline
    }
    pub fn is_editing(&self) -> bool {
        self.editor.is_some()
    }
    pub fn mark_saved(&mut self) {
        self.baseline = self.staged.clone();
    }

    /// The breadcrumb's level-2 node when a picker is open.
    pub fn editing_label(&self) -> Option<&'static str> {
        match &self.editor {
            Some(Editor::Quality(_)) => Some(s::P_QUALITY),
            Some(Editor::MaxRate(_)) => Some(s::P_MAX_RATE),
            Some(Editor::Retry(_)) => Some(s::P_RETRY_FAIL),
            None => None,
        }
    }
}
