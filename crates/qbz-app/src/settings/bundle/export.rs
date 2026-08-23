use serde_json::{Map, Value};

use super::error::BundleError;
use super::readers::{
    read_audio_settings, read_last_user_id, read_library_folders, read_playback_prefs,
    read_scrobblers, read_ui_prefs_streaming_quality,
};
use super::export_misc::{desktop_paths, hostname, now_rfc3339};
use super::token::load_decrypted_token;
use super::types::{Bundle, BundleSource, ExportOptions, ExportSource, ProfilePaths, SCHEMA_VERSION};

use crate::settings::daemon_prefs;

fn playback_to_json(p: &crate::settings::playback::PlaybackPreferences) -> Value {
    serde_json::to_value(p).unwrap_or(Value::Null)
}

/// Read a profile's settings into a [`Bundle`]. `--from desktop` reads the
/// GLOBAL desktop stores (the ONLY legal desktop-path access); `--from daemon`
/// reads the daemon roots. Domains the source cannot provide are ABSENT, never
/// empty objects (§2.9).
pub fn export(source: ExportSource, opts: &ExportOptions) -> Result<Bundle, BundleError> {
    let (paths, profile) = match &source {
        ExportSource::Desktop => (desktop_paths(), "desktop"),
        ExportSource::Daemon(p) => (
            ProfilePaths {
                config_root: p.config_root.clone(),
                data_root: p.data_root.clone(),
            },
            "daemon",
        ),
    };

    if matches!(source, ExportSource::Desktop) && !paths.data_root.exists() {
        return Err(BundleError::NoDesktopProfile);
    }

    let mut domains: Map<String, Value> = Map::new();

    // playback — PlaybackPreferences (global playback_preferences.db)
    if let Some(prefs) = read_playback_prefs(&paths.data_root) {
        domains.insert("playback".into(), playback_to_json(&prefs));
    }

    // audio — full-struct serde of AudioSettings (importer owns classification)
    if let Some(audio) = read_audio_settings(&paths.data_root) {
        if let Ok(v) = serde_json::to_value(&audio) {
            domains.insert("audio".into(), v);
        }
    }

    // prefs.streaming_quality — daemon_prefs (daemon) / ui_prefs.json (desktop)
    let streaming_quality = match &source {
        ExportSource::Daemon(_) => Some(daemon_prefs::load_at(&paths.data_root).streaming_quality),
        ExportSource::Desktop => read_ui_prefs_streaming_quality(&paths.data_root),
    };
    if let Some(q) = streaming_quality {
        let mut prefs = Map::new();
        prefs.insert("streaming_quality".into(), Value::String(q));
        domains.insert("prefs".into(), Value::Object(prefs));
    }

    // per-user domains — resolve the source uid.
    let uid = match &source {
        ExportSource::Desktop => crate::user_data::UserDataPaths::load_last_user_id(),
        ExportSource::Daemon(_) => read_last_user_id(&paths.data_root),
    };
    match uid {
        Some(uid) => {
            if let Some(scrob) = read_scrobblers(&paths.data_root, uid, opts.include_auth) {
                let mut integrations = Map::new();
                integrations.insert("scrobblers".into(), scrob);
                domains.insert("integrations".into(), Value::Object(integrations));
            }
            if let Some(folders) = read_library_folders(&paths.data_root) {
                domains.insert("library_folders".into(), folders);
            }
        }
        None => {
            log::info!(
                "[bundle] no last_user_id under this profile — per-user domains \
                 (integrations, library_folders) omitted"
            );
        }
    }

    // auth — SECRET, opt-in (§2.7). Export-side half of the double gate.
    if opts.include_auth {
        let token = load_decrypted_token(&source, &paths)?;
        if let Some(token) = token {
            let mut auth = Map::new();
            auth.insert("user_auth_token".into(), Value::String(token));
            if let Some(uid) = uid {
                auth.insert("user_id".into(), Value::Number(uid.into()));
            }
            domains.insert("auth".into(), Value::Object(auth));
        }
    }

    Ok(Bundle {
        schema_version: SCHEMA_VERSION,
        created_at: now_rfc3339(),
        source: BundleSource {
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            profile: profile.to_string(),
            hostname: hostname(),
        },
        domains,
    })
}
