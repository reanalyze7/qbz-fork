use crate::*;

pub(crate) fn wire_info_modals_suggestions_part4(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, _image_cache: &artwork::ImageCache, _settings_ctx: &Arc<settings::SettingsCtx>) {
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<PlaylistSuggestionsActions>()
            .on_play_track(move |track_id| {
                playlist_suggestions::play_track(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    track_id.to_string(),
                );
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<PlaylistSuggestionsActions>()
            .on_dismiss_track(move |track_id| {
                if let Some(w) = weak.upgrade() {
                    playlist_suggestions::dismiss_track(
                        &w,
                        runtime.clone(),
                        handle.clone(),
                        track_id.to_string(),
                    );
                }
            });
    }
    {
        // show-info / go-album / go-artist reuse the shared media-action arms:
        // ("track","track-info") opens the Track Info modal; ("album"/"artist",
        // "open") navigate — the same routing the playlist track rows use.
        let weak = window.as_weak();
        window
            .global::<PlaylistSuggestionsActions>()
            .on_show_info(move |track_id| {
                if let Some(w) = weak.upgrade() {
                    if !track_id.is_empty() {
                        w.invoke_media_action("track".into(), track_id, "track-info".into());
                    }
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<PlaylistSuggestionsActions>()
            .on_go_album(move |album_id| {
                if let Some(w) = weak.upgrade() {
                    if !album_id.is_empty() {
                        w.invoke_media_action("album".into(), album_id, "open".into());
                    }
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<PlaylistSuggestionsActions>()
            .on_go_artist(move |artist_id| {
                if let Some(w) = weak.upgrade() {
                    if !artist_id.is_empty() {
                        w.invoke_media_action("artist".into(), artist_id, "open".into());
                    }
                }
            });
    }

    // Artist Blacklist Manager actions (Task 11). Mutations are synchronous
    // (in-memory set + single SQLite ops via the artist_blacklist wrapper), so
    // no tokio handle is needed; each callback runs on the event-loop thread.
    {
        // open() — the forward-open seam (T10's Settings content-filtering row
        // calls this). Records the nav entry, swaps the view, then loads the
        // blacklist. Mirrors OfflineManagerActions.on_open.
        let weak = window.as_weak();
        window.global::<BlacklistActions>().on_open(move || {
            nav::record(nav::NavEntry::BlacklistManager);
            if let Some(w) = weak.upgrade() {
                w.global::<NavState>()
                    .set_view(ContentView::BlacklistManager);
                update_nav_flags(&w);
            }
            blacklist_manager::load(weak.clone());
        });
    }
}
