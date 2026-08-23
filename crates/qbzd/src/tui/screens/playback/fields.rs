use crate::tui::strings as s;

pub(super) const MAX_RATES: &[(&str, Option<u32>)] = &[
    (s::RATE_NO_LIMIT, None),
    ("44.1 kHz", Some(44_100)),
    ("48 kHz", Some(48_000)),
    ("88.2 kHz", Some(88_200)),
    ("96 kHz", Some(96_000)),
    ("176.4 kHz", Some(176_400)),
    ("192 kHz", Some(192_000)),
    ("352.8 kHz", Some(352_800)),
    ("384 kHz", Some(384_000)),
];

#[derive(Debug, Clone, PartialEq)]
pub struct StagedPlayback {
    pub quality: String,           // playback.quality
    pub limit_to_device: bool,     // audio.limit_quality_to_device
    pub max_sample_rate: Option<u32>, // audio.device_max_sample_rate
    pub allow_fallback: bool,      // audio.allow_quality_fallback
    pub fallback_behavior: String, // audio.quality_fallback_behavior
    pub autoplay: String,          // playback.autoplay
    pub gapless: bool,             // audio.gapless_enabled
    pub restore_session: bool,     // playback.persist_session
    pub resume_position: bool,     // playback.resume_playback_position
    pub mpris: bool,               // playback.mpris (applies on restart)
    /// Read-only, from the audio store — drives the Gapless disabled reason.
    pub streaming_only: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PField {
    Quality,
    Limit,
    MaxRate,
    AllowFallback,
    RetryFail,
    Continue,
    Gapless,
    Restore,
    Resume,
    Mpris,
}

/// `(shown, enabled, reason)` per §3.3.
pub fn row_state(field: PField, p: &StagedPlayback) -> (bool, bool, Option<&'static str>) {
    use PField::*;
    match field {
        MaxRate => (
            p.limit_to_device,
            true,
            if p.limit_to_device { None } else { Some(s::R_LIMIT_OFF) },
        ),
        Gapless => (
            true,
            !p.streaming_only,
            if p.streaming_only { Some(s::R_STREAMING_ONLY_ON) } else { None },
        ),
        Resume => (
            true,
            p.restore_session,
            if p.restore_session { None } else { Some(s::R_RESTORE_OFF) },
        ),
        _ => (true, true, None),
    }
}

pub fn visible_fields(p: &StagedPlayback) -> Vec<PField> {
    use PField::*;
    [Quality, Limit, MaxRate, AllowFallback, RetryFail, Continue, Gapless, Restore, Resume, Mpris]
        .into_iter()
        .filter(|f| row_state(*f, p).0)
        .collect()
}
