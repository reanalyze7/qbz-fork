use std::path::Path;

use serde::Deserialize;
use serde_json::{Map, Value};

use qbz_audio::settings::{AudioSettings, AudioSettingsStore};

use crate::settings::playback::{PlaybackPreferences, PlaybackPreferencesStore};
use crate::settings::scrobblers::ScrobblerSettingsStore;

// ============================ store readers (side-effect free) ============================

/// Read the current audio settings WITHOUT creating the DB (so `plan`/dry-run
/// never write). Returns `None` when the store file is absent (caller uses
/// `AudioSettings::default()`).
pub(super) fn read_audio_settings(data_root: &Path) -> Option<AudioSettings> {
    if !data_root.join("audio_settings.db").exists() {
        return None;
    }
    AudioSettingsStore::new_at(data_root)
        .and_then(|s| s.get_settings())
        .ok()
}

pub(super) fn read_playback_prefs(data_root: &Path) -> Option<PlaybackPreferences> {
    if !data_root.join("playback_preferences.db").exists() {
        return None;
    }
    PlaybackPreferencesStore::new_at(data_root)
        .and_then(|s| s.get_preferences())
        .ok()
}

pub(super) fn read_scrobblers(data_root: &Path, uid: u64, include_auth: bool) -> Option<Value> {
    let dir = data_root.join(format!("users/{uid}"));
    if !dir.join("scrobbler_settings.db").exists() {
        return None;
    }
    let store = ScrobblerSettingsStore::new_at(&dir).ok()?;
    let s = store.get_settings().ok()?;
    let mut obj = Map::new();
    obj.insert("enabled".into(), Value::Bool(s.enabled));
    obj.insert("lastfm_enabled".into(), Value::Bool(s.lastfm_enabled));
    obj.insert("lastfm_username".into(), Value::String(s.lastfm_username));
    obj.insert(
        "listenbrainz_enabled".into(),
        Value::Bool(s.listenbrainz_enabled),
    );
    obj.insert(
        "listenbrainz_username".into(),
        Value::String(s.listenbrainz_username),
    );
    // Secrets only with --include-auth; otherwise empty (§2.5).
    let lastfm_key = if include_auth { s.lastfm_session_key } else { String::new() };
    let lb_token = if include_auth { s.listenbrainz_token } else { String::new() };
    obj.insert("lastfm_session_key".into(), Value::String(lastfm_key));
    obj.insert("listenbrainz_token".into(), Value::String(lb_token));
    Some(Value::Object(obj))
}

pub(super) fn read_library_folders(data_root: &Path) -> Option<Value> {
    // Global desktop library DB (`<data>/qbz/library.db`); network_fs is
    // derived at runtime on the desktop and is cosmetic for the daemon (which
    // always skips this domain), so it is emitted false here.
    let db = data_root.join("library.db");
    if !db.exists() {
        return None;
    }
    let conn = rusqlite::Connection::open(&db).ok()?;
    let mut stmt = conn
        .prepare("SELECT path FROM library_folders ORDER BY path")
        .ok()?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .ok()?
        .filter_map(Result::ok);
    let arr: Vec<Value> = rows
        .map(|path| {
            let mut o = Map::new();
            o.insert("path".into(), Value::String(path));
            o.insert("network_fs".into(), Value::Bool(false));
            Value::Object(o)
        })
        .collect();
    if arr.is_empty() {
        None
    } else {
        Some(Value::Array(arr))
    }
}

pub(super) fn read_ui_prefs_streaming_quality(data_root: &Path) -> Option<String> {
    // Minimal serde read of ~90-field ui_prefs.json — serde ignores the rest
    // (§2.3; the 890-line ui_prefs.rs is NOT moved).
    #[derive(Deserialize)]
    struct MinimalUiPrefs {
        streaming_quality: Option<String>,
    }
    let text = std::fs::read_to_string(data_root.join("ui_prefs.json")).ok()?;
    let p: MinimalUiPrefs = serde_json::from_str(&text).ok()?;
    p.streaming_quality
}

// ---- last_user_id under an arbitrary root (daemon-safe) ----
pub(super) fn read_last_user_id(data_root: &Path) -> Option<u64> {
    std::fs::read_to_string(data_root.join("last_user_id"))
        .ok()?
        .trim()
        .parse::<u64>()
        .ok()
}

pub(crate) fn write_last_user_id(data_root: &Path, uid: u64) -> Result<(), String> {
    std::fs::create_dir_all(data_root).map_err(|e| e.to_string())?;
    std::fs::write(data_root.join("last_user_id"), uid.to_string()).map_err(|e| e.to_string())
}
