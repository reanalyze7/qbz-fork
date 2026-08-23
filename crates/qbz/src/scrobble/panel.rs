//! Panel init/toggle callbacks (bound in main.rs).

use slint::{ComponentHandle, Weak};

use crate::scrobbler_settings;
use crate::{AppWindow, ScrobbleState};

/// Panel init: seed `ScrobbleState` from the persisted store.
pub fn load(weak: Weak<AppWindow>) {
    let cfg = scrobbler_settings::get();
    let _ = weak.upgrade_in_event_loop(move |w| {
        let s = w.global::<ScrobbleState>();
        s.set_enabled(cfg.enabled);
        s.set_ui_collapsed(cfg.ui_collapsed);
        s.set_lastfm_enabled(cfg.lastfm_enabled);
        s.set_lastfm_authed(cfg.lastfm_is_authed());
        s.set_lastfm_username(cfg.lastfm_username.clone().into());
        s.set_lastfm_auth_url("".into());
        s.set_lastfm_busy(false);
        s.set_listenbrainz_enabled(cfg.listenbrainz_enabled);
        s.set_listenbrainz_authed(cfg.listenbrainz_is_authed());
        s.set_listenbrainz_username(cfg.listenbrainz_username.clone().into());
        s.set_listenbrainz_token_input("".into());
        s.set_listenbrainz_busy(false);
        s.set_status_text("".into());
        s.set_status_kind(0);
    });
}

pub fn enable_toggle(weak: Weak<AppWindow>, enabled: bool) {
    scrobbler_settings::set_enabled(enabled);
    let _ = weak.upgrade_in_event_loop(move |w| {
        w.global::<ScrobbleState>().set_enabled(enabled);
    });
}

pub fn collapse_toggle(collapsed: bool) {
    scrobbler_settings::set_ui_collapsed(collapsed);
}
