use crate::*;

pub(crate) fn wire_offline_and_auth_part3(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, _settings_ctx: &Arc<settings::SettingsCtx>) {


    // Open an album: record history, then load and show it.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window.on_open_album(move |album_id| {
            let album_id = album_id.to_string();
            // A local item carries a metadata group key, not a Qobuz id —
            // route it to the LocalAlbum view (Home "Recently played", the
            // now-playing bar's "Go to album", etc.) instead of the empty
            // Qobuz album view.
            if is_local_album_key(&album_id) {
                nav::record(nav::NavEntry::LocalAlbum(album_id.clone()));
                navigate_local_album(
                    runtime.clone(),
                    weak.clone(),
                    &handle,
                    image_cache.clone(),
                    album_id,
                );
            } else {
                // Feed Capa B if this Qobuz album was opened from the search
                // results page (gated inside the helper). Local-album keys take
                // the branch above and never reach here.
                if let Some(w) = weak.upgrade() {
                    record_search_interaction(
                        &w,
                        "album",
                        &album_id,
                        crate::search_service::InteractionAction::Open,
                    );
                }
                nav::record(nav::NavEntry::Album(album_id.clone()));
                navigate_album(
                    runtime.clone(),
                    weak.clone(),
                    &handle,
                    image_cache.clone(),
                    album_id,
                );
            }
            if let Some(w) = weak.upgrade() {
                update_nav_flags(&w);
            }
        });
    }

    // Open an artist: record history, then load and show the page.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window.on_open_artist(move |artist_ref| {
            let artist_ref = artist_ref.to_string();
            // Qobuz artists are numeric ids → the Qobuz artist page. Local
            // artists have no id, so their surfaces (LocalAlbum link, now-playing
            // "Go to artist") pass the NAME instead → the LocalLibrary Artists
            // tab, focused on that artist.
            if artist_ref.parse::<u64>().is_ok() {
                // Feed Capa B if this Qobuz artist was opened from the search
                // results page (gated inside the helper). Local artists
                // pass a NAME (non-numeric) and take the branch below — never
                // recorded.
                if let Some(w) = weak.upgrade() {
                    record_search_interaction(
                        &w,
                        "artist",
                        &artist_ref,
                        crate::search_service::InteractionAction::Open,
                    );
                }
                nav::record(nav::NavEntry::Artist(artist_ref.clone()));
                navigate_artist(
                    runtime.clone(),
                    weak.clone(),
                    &handle,
                    image_cache.clone(),
                    artist_ref,
                );
                if let Some(w) = weak.upgrade() {
                    update_nav_flags(&w);
                }
            } else if !artist_ref.trim().is_empty() {
                open_local_artist(&runtime, &weak, &handle, &image_cache, artist_ref);
            }
        });
    }
}
