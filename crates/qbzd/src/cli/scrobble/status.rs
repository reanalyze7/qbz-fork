use crate::paths::ProfileRoots;

use super::support::{not_connected, nudge_reload, open_store};

/// `qbzd scrobble status` — per-provider connection + enabled state.
pub fn status(roots: &ProfileRoots) -> i32 {
    let store = match open_store(roots) {
        Ok(s) => s,
        Err(code) => return code,
    };
    let s = match store.get_settings() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: {e}");
            return 1;
        }
    };
    println!(
        "last.fm       : {}",
        provider_line(s.lastfm_is_authed(), s.lastfm_active(), &s.lastfm_username)
    );
    println!(
        "listenbrainz  : {}",
        provider_line(s.listenbrainz_is_authed(), s.listenbrainz_active(), &s.listenbrainz_username)
    );
    0
}

/// `qbzd scrobble enable|disable <lastfm|listenbrainz>` — keep the credentials
/// but start/stop scrobbling to that provider.
pub async fn set_enabled(host: Option<String>, provider: String, enabled: bool, roots: &ProfileRoots) -> i32 {
    let store = match open_store(roots) {
        Ok(s) => s,
        Err(code) => return code,
    };
    let s = match store.get_settings() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: {e}");
            return 1;
        }
    };
    let result = match provider.as_str() {
        "lastfm" if !s.lastfm_is_authed() => return not_connected(&provider),
        "listenbrainz" if !s.listenbrainz_is_authed() => return not_connected(&provider),
        "lastfm" => store.set_lastfm_enabled(enabled),
        "listenbrainz" => store.set_listenbrainz_enabled(enabled),
        other => {
            eprintln!("error: unknown provider '{other}'");
            eprintln!("  → lastfm | listenbrainz");
            return 2;
        }
    };
    if let Err(e) = result {
        eprintln!("error: {e}");
        return 1;
    }
    // Enabling a provider also lifts the master toggle (off = nothing scrobbles).
    if enabled {
        let _ = store.set_enabled(true);
    }
    nudge_reload(host).await;
    println!("{provider} scrobbling {}", if enabled { "enabled" } else { "disabled" });
    0
}

fn provider_line(authed: bool, active: bool, name: &str) -> String {
    match (authed, active) {
        (false, _) => "not connected".to_string(),
        (true, true) => format!("on as {name}"),
        (true, false) => format!("off (connected as {name})"),
    }
}
