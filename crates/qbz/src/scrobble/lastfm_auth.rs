//! Last.fm step 1 (request token + open browser) and the enable toggle.

use slint::{ComponentHandle, Weak};

use qbz_integrations::LastFmClient;

use crate::scrobbler_settings;
use crate::{AppWindow, ScrobbleState};

use super::{set_status, LASTFM_PENDING_TOKEN};

pub fn lastfm_enable_toggle(weak: Weak<AppWindow>, enabled: bool) {
    scrobbler_settings::set_lastfm_enabled(enabled);
    let _ = weak.upgrade_in_event_loop(move |w| {
        w.global::<ScrobbleState>().set_lastfm_enabled(enabled);
    });
}

/// Step 1: request a token, open the Last.fm authorize URL in the browser, and
/// reveal the "Finish" affordance. Mirrors the Svelte `v2_lastfm_get_auth_url`
/// + open path (system browser).
pub fn lastfm_connect(weak: Weak<AppWindow>, handle: tokio::runtime::Handle) {
    let _ = weak.upgrade_in_event_loop(|w| w.global::<ScrobbleState>().set_lastfm_busy(true));
    handle.spawn(async move {
        let client = LastFmClient::new();
        match client.get_token().await {
            Ok((token, auth_url)) => {
                if let Ok(mut g) = LASTFM_PENDING_TOKEN.lock() {
                    *g = Some(token);
                }
                let auth_url_ui = auth_url.clone();
                let _ = weak.upgrade_in_event_loop(move |w| {
                    let s = w.global::<ScrobbleState>();
                    s.set_lastfm_busy(false);
                    s.set_lastfm_auth_url(auth_url_ui.into());
                });
                // Open the browser to authorize.
                if let Err(e) = open::that(&auth_url) {
                    log::warn!("[qbz-slint] open Last.fm auth url failed: {e}");
                }
                set_status(
                    &weak,
                    qbz_i18n::t("Authorize Qoqobuz in your browser, then click \"Finish\""),
                    1,
                );
            }
            Err(e) => {
                let _ = weak.upgrade_in_event_loop(|w| {
                    w.global::<ScrobbleState>().set_lastfm_busy(false);
                });
                set_status(&weak, qbz_i18n::t_args("Error: {}", &[&e.to_string()]), 3);
                crate::toast::error_weak(&weak, qbz_i18n::t("Last.fm sign-in failed to start"));
            }
        }
    });
}

/// Re-open the stored authorize URL (in case the browser did not launch).
pub fn lastfm_open_auth_url(weak: Weak<AppWindow>) {
    let url = weak
        .upgrade()
        .map(|w| w.global::<ScrobbleState>().get_lastfm_auth_url().to_string())
        .unwrap_or_default();
    if url.is_empty() {
        return;
    }
    if let Err(e) = open::that(&url) {
        log::warn!("[qbz-slint] open Last.fm auth url failed: {e}");
    }
}
