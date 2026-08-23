//! Last.fm step 2 (exchange the pending token for a session) and disconnect.

use slint::{ComponentHandle, Weak};

use qbz_integrations::LastFmClient;

use crate::scrobbler_settings;
use crate::{AppWindow, ScrobbleState};

use super::{set_status, LASTFM_PENDING_TOKEN};

/// Step 2 (the user clicked "Finish" after authorizing): exchange the pending
/// token for a session key + username and persist it. Mirrors
/// `v2_lastfm_complete_auth`.
pub fn lastfm_confirm(weak: Weak<AppWindow>, handle: tokio::runtime::Handle) {
    let token = LASTFM_PENDING_TOKEN.lock().ok().and_then(|g| g.clone());
    let Some(token) = token else {
        set_status(&weak, qbz_i18n::t("Start the sign-in first"), 3);
        return;
    };
    let _ = weak.upgrade_in_event_loop(|w| w.global::<ScrobbleState>().set_lastfm_busy(true));
    handle.spawn(async move {
        let mut client = LastFmClient::new();
        match client.get_session(&token).await {
            Ok(session) => {
                scrobbler_settings::set_lastfm_session(&session.key, &session.name);
                // Default the per-service flag ON the first time we connect, so
                // scrobbling starts without an extra toggle.
                if !scrobbler_settings::get().lastfm_enabled {
                    scrobbler_settings::set_lastfm_enabled(true);
                }
                if let Ok(mut g) = LASTFM_PENDING_TOKEN.lock() {
                    *g = None;
                }
                let username = session.name.clone();
                let _ = weak.upgrade_in_event_loop(move |w| {
                    let s = w.global::<ScrobbleState>();
                    s.set_lastfm_busy(false);
                    s.set_lastfm_authed(true);
                    s.set_lastfm_enabled(true);
                    s.set_lastfm_username(username.into());
                    s.set_lastfm_auth_url("".into());
                });
                set_status(&weak, qbz_i18n::t_args("Connected as {}", &[session.name.as_str()]), 2);
            }
            Err(e) => {
                let _ = weak.upgrade_in_event_loop(|w| {
                    w.global::<ScrobbleState>().set_lastfm_busy(false);
                });
                set_status(
                    &weak,
                    qbz_i18n::t_args("Error: {} (did you authorize in the browser?)", &[&e.to_string()]),
                    3,
                );
            }
        }
    });
}

pub fn lastfm_disconnect(weak: Weak<AppWindow>) {
    scrobbler_settings::disconnect_lastfm();
    if let Ok(mut g) = LASTFM_PENDING_TOKEN.lock() {
        *g = None;
    }
    let _ = weak.upgrade_in_event_loop(|w| {
        let s = w.global::<ScrobbleState>();
        s.set_lastfm_authed(false);
        s.set_lastfm_username("".into());
        s.set_lastfm_auth_url("".into());
        s.set_lastfm_busy(false);
    });
    set_status(&weak, qbz_i18n::t("Last.fm disconnected"), 1);
}
