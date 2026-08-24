use crate::*;

/// Read the current DiscoverBrowse "View all" target when that page is
/// the active view, so a genre-filter change can re-navigate it instead
/// of the Discover home index (the selection is shared across surfaces).
/// Returns None when any other view is showing. UI thread only.
pub(crate) fn current_browse_target(window: &AppWindow) -> Option<(String, String)> {
    if window.global::<NavState>().get_view() != ContentView::DiscoverBrowse {
        return None;
    }
    let state = window.global::<DiscoverBrowseState>();
    let endpoint = state.get_endpoint().to_string();
    if endpoint.is_empty() {
        return None;
    }
    Some((endpoint, state.get_title().to_string()))
}

/// Whether the Qobuz Playlists "View all" page is the active view, so a
/// genre-filter change re-navigates it (preserving its selected tag)
/// instead of reloading the Discover home index. UI thread only.
pub(crate) fn current_playlist_browse_showing(window: &AppWindow) -> bool {
    window.global::<NavState>().get_view() == ContentView::PlaylistBrowse
}

/// Push the navigation history flags onto `NavState`. UI thread only.
pub(crate) fn update_nav_flags(window: &AppWindow) {
    let state = window.global::<NavState>();
    state.set_can_back(nav::can_back());
    state.set_can_forward(nav::can_forward());
}

/// Navigate to the entity a resolved "Open Qobuz Link" points at. Albums /
/// artists / playlists open their detail view directly; a track is fetched to
/// resolve its album, then that album opens (mirrors the Tauri behavior).
pub(crate) fn apply_resolved_link(
    link: qbz_music_link::ResolvedLink,
    runtime: &Arc<AppRuntime<SlintAdapter>>,
    weak: &slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: &artwork::ImageCache,
) {
    use qbz_music_link::ResolvedLink;
    match link {
        ResolvedLink::OpenAlbum(id) => {
            navigate_album(runtime.clone(), weak.clone(), handle, image_cache.clone(), id);
        }
        ResolvedLink::OpenArtist(id) => {
            navigate_artist(
                runtime.clone(),
                weak.clone(),
                handle,
                image_cache.clone(),
                id.to_string(),
            );
        }
        ResolvedLink::OpenPlaylist(id) => {
            navigate_playlist(
                runtime.clone(),
                weak.clone(),
                handle,
                image_cache.clone(),
                id.to_string(),
            );
        }
        ResolvedLink::OpenTrack(id) => {
            let runtime = runtime.clone();
            let weak = weak.clone();
            let handle = handle.clone();
            let image_cache = image_cache.clone();
            handle.clone().spawn(async move {
                match runtime.core().get_track(id).await {
                    Ok(track) => {
                        if let Some(album_id) = track.album.as_ref().map(|a| a.id.clone()) {
                            navigate_album(
                                runtime.clone(),
                                weak.clone(),
                                &handle,
                                image_cache.clone(),
                                album_id,
                            );
                        }
                    }
                    Err(e) => {
                        log::error!("[qbz-slint] open-link: get_track failed: {e}");
                    }
                }
            });
        }
    }
}

