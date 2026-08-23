use std::path::Path;

use serde_json::Value;

use crate::settings::daemon_prefs;
use crate::settings::playback::PlaybackPreferencesStore;
use crate::settings::scrobblers::ScrobblerSettingsStore;

use super::types::ProfilePaths;

pub(super) use super::apply_writes_audio::apply_audio_writes;

pub(super) fn as_bool(v: &Value) -> bool {
    v.as_bool().unwrap_or(false)
}

pub(super) fn apply_playback_writes(data_root: &Path, writes: &[(&str, &Value)]) -> Result<(), String> {
    let store = PlaybackPreferencesStore::new_at(data_root)?;
    for (key, value) in writes {
        match *key {
            "autoplay_mode" => {
                let mode = serde_json::from_value((*value).clone())
                    .map_err(|e| format!("autoplay_mode: {e}"))?;
                store.set_autoplay_mode(mode)?;
            }
            "show_context_icon" => store.set_show_context_icon(as_bool(value))?,
            "persist_session" => store.set_persist_session(as_bool(value))?,
            "resume_playback_position" => store.set_resume_playback_position(as_bool(value))?,
            other => log::warn!("[bundle] apply: unhandled playback key {other}"),
        }
    }
    Ok(())
}

pub(super) fn apply_prefs_quality(data_root: &Path, value: &Value) -> Result<(), String> {
    let mut prefs = daemon_prefs::load_at(data_root);
    if let Some(q) = value.as_str() {
        prefs.streaming_quality = q.to_string();
    }
    daemon_prefs::save_at(&prefs, data_root)
}

pub(super) fn apply_scrobbler_writes(
    data_root: &Path,
    uid: u64,
    writes: &[(&str, &Value)],
) -> Result<(), String> {
    let dir = data_root.join(format!("users/{uid}"));
    let store = ScrobblerSettingsStore::new_at(&dir)?;
    let mut current = store.get_settings()?;
    for (key, value) in writes {
        match *key {
            "enabled" => store.set_enabled(as_bool(value))?,
            "lastfm_enabled" => store.set_lastfm_enabled(as_bool(value))?,
            "lastfm_username" => {
                current.lastfm_username = value.as_str().unwrap_or("").to_string();
                store.set_lastfm_session(&current.lastfm_session_key, &current.lastfm_username)?;
            }
            "lastfm_session_key" => {
                current.lastfm_session_key = value.as_str().unwrap_or("").to_string();
                store.set_lastfm_session(&current.lastfm_session_key, &current.lastfm_username)?;
            }
            "listenbrainz_enabled" => store.set_listenbrainz_enabled(as_bool(value))?,
            "listenbrainz_username" => {
                current.listenbrainz_username = value.as_str().unwrap_or("").to_string();
                store.set_listenbrainz_token(
                    &current.listenbrainz_token,
                    &current.listenbrainz_username,
                )?;
            }
            "listenbrainz_token" => {
                current.listenbrainz_token = value.as_str().unwrap_or("").to_string();
                store.set_listenbrainz_token(
                    &current.listenbrainz_token,
                    &current.listenbrainz_username,
                )?;
            }
            other => log::warn!("[bundle] apply: unhandled scrobbler key {other}"),
        }
    }
    Ok(())
}

pub(super) fn persist_auth(target: &ProfilePaths, token: &str, uid: u64) -> Result<(), String> {
    qbz_credentials::save_oauth_token_at(&target.config_root, token)?;
    // last_user_id under the DAEMON root (NEVER the desktop global path — the
    // daemon must not touch ~/.local/share/qbz; 04 §5.7 cites the desktop fn
    // only for the flat-file format).
    super::readers::write_last_user_id(&target.data_root, uid)?;
    std::fs::create_dir_all(target.data_root.join(format!("users/{uid}")))
        .map_err(|e| e.to_string())?;
    Ok(())
}
