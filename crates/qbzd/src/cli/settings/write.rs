// crates/qbzd/src/cli/settings/write.rs — `SetError` + the validated
// per-key writer that dispatches to the domain-specific arms
// (`write_audio_reinit.rs`/`write_audio_reload.rs`/the playback arms below).

use qbz_app::settings::daemon_prefs;

use crate::paths::ProfileRoots;

use super::codec_bool::parse_bool;
use super::codec_playback::{parse_autoplay, parse_streaming_quality};
use super::keys::{classify, unknown_key_error, ApplyClass};
use super::store::open_playback;
use super::write_audio_reinit::write_audio_reinit;
use super::write_audio_reload::write_audio_reload;

/// The two exit-code classes a [`write_one`] failure can fall into (02 §1.3,
/// the frozen exit-code table: 2 is reserved for USAGE mistakes only). An
/// unknown key or an invalid value for a known key never touches a store —
/// that is `Usage` (exit 2). A key that classified and parsed fine but whose
/// backing store then failed to open or write — a disk-full/permissions/
/// corrupt-file problem — is not a usage mistake: that is `Io` (exit 1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SetError {
    Usage(String),
    Io(String),
}

impl SetError {
    pub(super) fn message(&self) -> &str {
        match self {
            SetError::Usage(m) | SetError::Io(m) => m,
        }
    }
}

impl std::fmt::Display for SetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message())
    }
}

/// Validate + write ONE canonical key. Returns its [`ApplyClass`] on success
/// (the CLI's own success-line hint — see module doc). `pub(crate)` so the T13
/// setup TUI persists every screen through this SAME validated writer (03 §6 —
/// the TUI adds no persistence of its own). Every arm parses (`Usage` on
/// failure) BEFORE it opens/writes a store (`Io` on failure) — see [`SetError`].
pub(crate) fn write_one(roots: &ProfileRoots, key: &str, raw: &str) -> Result<ApplyClass, SetError> {
    let Some(class) = classify(key) else {
        return Err(SetError::Usage(unknown_key_error(key)));
    };
    if write_audio_reinit(roots, key, raw)? {
        return Ok(class);
    }
    if write_audio_reload(roots, key, raw)? {
        return Ok(class);
    }
    if write_playback(roots, key, raw)? {
        return Ok(class);
    }
    unreachable!("KEY_TABLE/write_one drifted apart on key: {key}");
}

/// The `playback.*` arms (daemon_prefs + `PlaybackPreferencesStore`).
fn write_playback(roots: &ProfileRoots, key: &str, raw: &str) -> Result<bool, SetError> {
    match key {
        "playback.quality" => {
            let v = parse_streaming_quality(raw).map_err(SetError::Usage)?;
            let mut prefs = daemon_prefs::load_at(&roots.data);
            prefs.streaming_quality = v;
            daemon_prefs::save_at(&prefs, &roots.data).map_err(SetError::Io)?;
        }
        "playback.mpris" => {
            let v = parse_bool(raw).map_err(SetError::Usage)?;
            let mut prefs = daemon_prefs::load_at(&roots.data);
            prefs.mpris_enabled = v;
            daemon_prefs::save_at(&prefs, &roots.data).map_err(SetError::Io)?;
        }
        "playback.autoplay" => {
            let v = parse_autoplay(raw).map_err(SetError::Usage)?;
            open_playback(roots).map_err(SetError::Io)?.set_autoplay_mode(v).map_err(SetError::Io)?
        }
        "playback.persist_session" => {
            let v = parse_bool(raw).map_err(SetError::Usage)?;
            open_playback(roots)
                .map_err(SetError::Io)?
                .set_persist_session(v)
                .map_err(SetError::Io)?
        }
        "playback.resume_playback_position" => {
            let v = parse_bool(raw).map_err(SetError::Usage)?;
            open_playback(roots)
                .map_err(SetError::Io)?
                .set_resume_playback_position(v)
                .map_err(SetError::Io)?
        }
        "playback.show_context_icon" => {
            let v = parse_bool(raw).map_err(SetError::Usage)?;
            open_playback(roots)
                .map_err(SetError::Io)?
                .set_show_context_icon(v)
                .map_err(SetError::Io)?
        }
        _ => return Ok(false),
    }
    Ok(true)
}
