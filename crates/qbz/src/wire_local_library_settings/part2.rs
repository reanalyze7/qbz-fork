use crate::*;

pub(crate) fn wire_local_library_settings_part2(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {
    {
        let weak = window.as_weak();
        window
            .global::<LibraryManageActions>()
            .on_set_filter(move |_q| local_library_settings::set_filter(weak.clone()));
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LibraryManageActions>()
            .on_scan_all(move || local_library_settings::scan_all(weak.clone(), handle.clone()));
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LibraryManageActions>()
            .on_scan_folder(move |id| {
                local_library_settings::scan_folder(weak.clone(), handle.clone(), id as i64)
            });
    }
    {
        window
            .global::<LibraryManageActions>()
            .on_stop_scan(move || local_library_settings::stop_scan());
    }

    // Settings > Integrations — scrobblers (Last.fm + ListenBrainz). The auth
    // flows + the now-playing/scrobble fire live in `scrobble`; the persisted
    // store is the per-user `scrobbler_settings.db`.
    {
        let weak = window.as_weak();
        window
            .global::<ScrobbleActions>()
            .on_load(move || scrobble::load(weak.clone()));
    }
    {
        let weak = window.as_weak();
        window
            .global::<ScrobbleActions>()
            .on_enable_toggle(move |b| scrobble::enable_toggle(weak.clone(), b));
    }
    {
        window
            .global::<ScrobbleActions>()
            .on_collapse_toggle(move |b| scrobble::collapse_toggle(b));
    }
    {
        let weak = window.as_weak();
        window
            .global::<ScrobbleActions>()
            .on_lastfm_enable_toggle(move |b| scrobble::lastfm_enable_toggle(weak.clone(), b));
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<ScrobbleActions>()
            .on_lastfm_connect(move || scrobble::lastfm_connect(weak.clone(), handle.clone()));
    }
    {
        let weak = window.as_weak();
        window
            .global::<ScrobbleActions>()
            .on_lastfm_open_auth_url(move || scrobble::lastfm_open_auth_url(weak.clone()));
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<ScrobbleActions>()
            .on_lastfm_confirm(move || scrobble::lastfm_confirm(weak.clone(), handle.clone()));
    }
    {
        let weak = window.as_weak();
        window
            .global::<ScrobbleActions>()
            .on_lastfm_disconnect(move || scrobble::lastfm_disconnect(weak.clone()));
    }
    {
        let weak = window.as_weak();
        window
            .global::<ScrobbleActions>()
            .on_listenbrainz_enable_toggle(move |b| {
                scrobble::listenbrainz_enable_toggle(weak.clone(), b)
            });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<ScrobbleActions>()
            .on_listenbrainz_set_token(move |tok| {
                scrobble::listenbrainz_set_token(weak.clone(), handle.clone(), tok.to_string())
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<ScrobbleActions>()
            .on_listenbrainz_disconnect(move || scrobble::listenbrainz_disconnect(weak.clone()));
    }

    // Tag editor (local album metadata) — open via on_media_action("album",
    // "edit"); these wire the modal's own actions.
    {
        let weak = window.as_weak();
        window
            .global::<TagEditorActions>()
            .on_close(move || tag_editor::close_tag_editor(weak.clone()));
    }
}
