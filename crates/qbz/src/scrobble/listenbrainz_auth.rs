//! ListenBrainz token connect/disconnect.

use slint::{ComponentHandle, Weak};

use qbz_integrations::listenbrainz::cache::ListenBrainzCache;
use qbz_integrations::ListenBrainzClient;

use crate::scrobbler_settings;
use crate::{AppWindow, ScrobbleState};

use super::{listenbrainz_cache_path, rt_handle, set_status};

pub fn listenbrainz_enable_toggle(weak: Weak<AppWindow>, enabled: bool) {
    scrobbler_settings::set_listenbrainz_enabled(enabled);
    let _ = weak.upgrade_in_event_loop(move |w| {
        w.global::<ScrobbleState>().set_listenbrainz_enabled(enabled);
    });
}

/// Save + validate a ListenBrainz user token. Mirrors `v2_listenbrainz_connect`
/// — validated against `/validate-token`, then persisted to this build's store
/// AND the shared `ListenBrainzCache` (so the Tauri build picks it up).
pub fn listenbrainz_set_token(weak: Weak<AppWindow>, handle: tokio::runtime::Handle, token: String) {
    let token = token.trim().to_string();
    if token.is_empty() {
        set_status(&weak, qbz_i18n::t("Paste your ListenBrainz user token first"), 3);
        return;
    }
    let _ = weak.upgrade_in_event_loop(|w| {
        w.global::<ScrobbleState>().set_listenbrainz_busy(true)
    });
    handle.spawn(async move {
        let client = ListenBrainzClient::new();
        match client.set_token(&token).await {
            Ok(info) => {
                scrobbler_settings::set_listenbrainz_token(&token, &info.user_name);
                if !scrobbler_settings::get().listenbrainz_enabled {
                    scrobbler_settings::set_listenbrainz_enabled(true);
                }
                // Write-through to the shared cache (Tauri reads it at session
                // start). Best-effort.
                if let Some(path) = listenbrainz_cache_path() {
                    let tok = token.clone();
                    let name = info.user_name.clone();
                    let _ = tokio::task::spawn_blocking(move || {
                        ListenBrainzCache::new(&path)
                            .and_then(|c| c.save_credentials(&tok, &name))
                    })
                    .await;
                }
                let username = info.user_name.clone();
                let _ = weak.upgrade_in_event_loop(move |w| {
                    let s = w.global::<ScrobbleState>();
                    s.set_listenbrainz_busy(false);
                    s.set_listenbrainz_authed(true);
                    s.set_listenbrainz_enabled(true);
                    s.set_listenbrainz_username(username.into());
                    s.set_listenbrainz_token_input("".into());
                });
                set_status(&weak, qbz_i18n::t_args("Connected as {}", &[info.user_name.as_str()]), 2);
            }
            Err(e) => {
                let _ = weak.upgrade_in_event_loop(|w| {
                    w.global::<ScrobbleState>().set_listenbrainz_busy(false);
                });
                set_status(&weak, qbz_i18n::t_args("Error: {}", &[&e.to_string()]), 3);
            }
        }
    });
}

pub fn listenbrainz_disconnect(weak: Weak<AppWindow>) {
    scrobbler_settings::disconnect_listenbrainz();
    // Clear the shared cache credentials too (mirrors Tauri's disconnect).
    if let Some(path) = listenbrainz_cache_path() {
        if let Some(handle) = rt_handle() {
            handle.spawn(async move {
                let _ = tokio::task::spawn_blocking(move || {
                    ListenBrainzCache::new(&path).and_then(|c| c.clear_credentials())
                })
                .await;
            });
        }
    }
    let _ = weak.upgrade_in_event_loop(|w| {
        let s = w.global::<ScrobbleState>();
        s.set_listenbrainz_authed(false);
        s.set_listenbrainz_username("".into());
        s.set_listenbrainz_token_input("".into());
        s.set_listenbrainz_busy(false);
    });
    set_status(&weak, qbz_i18n::t("ListenBrainz disconnected"), 1);
}
