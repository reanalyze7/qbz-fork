// ============================ mappers ============================
use qbz_app::settings::playback::AutoplayMode;

use crate::tui::strings as s;

use super::fields::MAX_RATES;

pub(super) fn quality_label(q: &str) -> &'static str {
    match q {
        "mp3" => s::Q_MP3,
        "cd" => s::Q_CD,
        "hires" => s::Q_HIRES,
        _ => s::Q_HIRES_PLUS,
    }
}

pub(super) fn max_rate_label(v: Option<u32>) -> &'static str {
    MAX_RATES
        .iter()
        .find(|(_, r)| *r == v)
        .map(|(l, _)| *l)
        .unwrap_or(s::RATE_NO_LIMIT)
}

pub(super) fn retry_label(v: &str) -> &'static str {
    match v {
        "always_skip" => s::RETRY_SKIP,
        "always_fallback" => s::RETRY_FALLBACK,
        _ => s::RETRY_ASK, // stored `ask` — rendered until the operator picks
    }
}

pub(super) fn autoplay_label(v: &str) -> &'static str {
    match v {
        "track_only" => s::AUTOPLAY_OFF,
        "infinite" => s::AUTOPLAY_INFINITE,
        _ => s::AUTOPLAY_ON,
    }
}

pub(super) fn autoplay_value(mode: AutoplayMode) -> &'static str {
    match mode {
        AutoplayMode::ContinueWithinSource => "continue",
        AutoplayMode::PlayTrackOnly => "track_only",
        AutoplayMode::InfiniteRadio => "infinite",
    }
}
