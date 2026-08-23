use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoplayMode {
    /// Continue playing within the source (album, playlist, etc.)
    #[serde(rename = "continue")]
    ContinueWithinSource,
    /// Play only the selected track, then stop.
    #[serde(rename = "track_only")]
    PlayTrackOnly,
    /// Create infinite radio when queue ends (based on recent tracks).
    #[serde(rename = "infinite")]
    InfiniteRadio,
}

impl Default for AutoplayMode {
    fn default() -> Self {
        Self::ContinueWithinSource
    }
}

impl AutoplayMode {
    pub(super) fn to_db_value(self) -> &'static str {
        match self {
            AutoplayMode::ContinueWithinSource => "continue",
            AutoplayMode::PlayTrackOnly => "track_only",
            AutoplayMode::InfiniteRadio => "infinite",
        }
    }

    pub(super) fn from_db_value(value: &str) -> Self {
        match value {
            "track_only" => AutoplayMode::PlayTrackOnly,
            "infinite" => AutoplayMode::InfiniteRadio,
            _ => AutoplayMode::ContinueWithinSource,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackPreferences {
    pub autoplay_mode: AutoplayMode,
    /// Portable UI preference for showing the context-stack icon.
    /// This is not domain playback behavior.
    pub show_context_icon: bool,
    pub persist_session: bool,
    /// Sub-preference of `persist_session`. When true, restoring a
    /// session also seeks to `current_position_secs` of the saved
    /// track. When false (default), the saved track is shown paused at
    /// 0:00 and the user starts the next listen fresh.
    pub resume_playback_position: bool,
}

impl Default for PlaybackPreferences {
    fn default() -> Self {
        Self {
            autoplay_mode: AutoplayMode::ContinueWithinSource,
            // Playback preferences are opt-out: on by default.
            show_context_icon: true,
            persist_session: true,
            resume_playback_position: true,
        }
    }
}
