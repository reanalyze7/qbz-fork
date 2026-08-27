use std::io::{BufRead, Write};

use crate::paths::ProfileRoots;

use super::support::{nudge_reload, open_store};

/// `qbzd scrobble login lastfm` — the Last.fm web-auth flow (print URL →
/// user approves → exchange for a session key), mirroring `qbzd login`.
pub async fn login_lastfm(host: Option<String>, roots: &ProfileRoots) -> i32 {
    let mut client = qbz_integrations::lastfm::LastFmClient::new();
    let (token, auth_url) = match client.get_token().await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: Last.fm token request failed: {e}");
            eprintln!("  → check your connection and retry");
            return 1;
        }
    };
    println!("Authorize Qoqobuz on Last.fm, then come back here:");
    println!("  {auth_url}");
    print!("Press Enter after you've clicked \"Yes, allow access\"… ");
    let _ = std::io::stdout().flush();
    let mut line = String::new();
    let _ = std::io::stdin().lock().read_line(&mut line);

    let session = match client.get_session(&token).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: Last.fm authorization not completed: {e}");
            eprintln!("  → approve access on the page first, then run this again");
            return 1;
        }
    };
    qbz_log::register_secret(session.key.clone());

    let store = match open_store(roots) {
        Ok(s) => s,
        Err(code) => return code,
    };
    if let Err(e) = store
        .set_lastfm_session(&session.key, &session.name)
        .and_then(|_| store.set_lastfm_enabled(true))
        .and_then(|_| store.set_enabled(true))
    {
        eprintln!("error: {e}");
        return 1;
    }
    nudge_reload(host).await;
    println!("Last.fm connected as {} — scrobbling enabled", session.name);
    0
}

/// `qbzd scrobble login listenbrainz --token <TOKEN>` — validate and store a
/// ListenBrainz user token (from listenbrainz.org/settings).
pub async fn login_listenbrainz(host: Option<String>, token: String, roots: &ProfileRoots) -> i32 {
    let client = qbz_integrations::listenbrainz::ListenBrainzClient::new();
    let info = match client.set_token(&token).await {
        Ok(i) => i,
        Err(e) => {
            eprintln!("error: ListenBrainz token rejected: {e}");
            eprintln!("  → get a token at https://listenbrainz.org/settings/");
            return 1;
        }
    };
    qbz_log::register_secret(token.clone());

    let store = match open_store(roots) {
        Ok(s) => s,
        Err(code) => return code,
    };
    if let Err(e) = store
        .set_listenbrainz_token(&token, &info.user_name)
        .and_then(|_| store.set_listenbrainz_enabled(true))
        .and_then(|_| store.set_enabled(true))
    {
        eprintln!("error: {e}");
        return 1;
    }
    nudge_reload(host).await;
    println!("ListenBrainz connected as {} — scrobbling enabled", info.user_name);
    0
}
