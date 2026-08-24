use crate::*;

/// Ctrl/Cmd+A handler: select-all (NEVER toggles to clear — 1:1 Tauri
/// `isSelectAllShortcut`) in whichever track surface is showing AND has its
/// multi-select mode ON. Returns true iff a surface consumed it. Routed by
/// `NavState.view`, mirroring the central toggle-select arm. LocalLibrary
/// Tracks / Offline / Mix / Label route through their own paths (slices 3-5).
pub(crate) fn select_all_active_surface(window: &AppWindow) -> bool {
    let view = window.global::<NavState>().get_view();
    // Offline Manager is always-on selection over OfflineRow (not TrackItem) —
    // Ctrl+A always selects all there.
    if view == ContentView::OfflineManager {
        offline_manager::set_all_selected(window, true);
        return true;
    }
    // LocalLibrary Albums tab selects AlbumCardItem (not TrackItem) — its own
    // select-all-only path. The Tracks tab falls through to the TrackItem match.
    if view == ContentView::LocalLibrary
        && window.global::<LocalLibraryState>().get_active_tab().as_str() == "albums"
    {
        if window.global::<LocalLibraryState>().get_albums_multi_select() {
            local_library::select_all_albums_only(window);
            return true;
        }
        return false;
    }
    let (model, on): (slint::ModelRc<TrackItem>, bool) = match view {
        ContentView::Album => (
            window.global::<AlbumState>().get_tracks(),
            window.global::<AlbumState>().get_multi_select(),
        ),
        ContentView::Artist => (
            window.global::<ArtistState>().get_top_tracks(),
            window.global::<ArtistState>().get_top_tracks_multi_select(),
        ),
        ContentView::Playlist => (
            window.global::<PlaylistState>().get_tracks(),
            window.global::<PlaylistState>().get_multi_select_mode(),
        ),
        ContentView::Favorites => (
            window.global::<FavoritesState>().get_tracks_visible(),
            window.global::<FavoritesState>().get_tracks_multi_select(),
        ),
        ContentView::LocalLibrary => (
            window.global::<LocalLibraryState>().get_tracks_visible(),
            window.global::<LocalLibraryState>().get_tracks_multi_select(),
        ),
        ContentView::Mix => (
            window.global::<MixState>().get_tracks(),
            window.global::<MixState>().get_multi_select(),
        ),
        ContentView::Label => (
            window.global::<LabelState>().get_top_tracks(),
            window.global::<LabelState>().get_multi_select(),
        ),
        _ => return false,
    };
    if !on {
        return false;
    }
    if let Some(vm) = model.as_any().downcast_ref::<slint::VecModel<TrackItem>>() {
        selection::select_all(vm, |t, v| t.selected = v);
    }
    match view {
        ContentView::Album => album::recount_selected(window),
        ContentView::Artist => artist::recount_selected(window),
        ContentView::Playlist => playlist::recount_selected(window),
        ContentView::Favorites => favorites::recount_selected(window),
        ContentView::LocalLibrary => local_library::recount_tracks_selected(window),
        ContentView::Mix => mix::recount_selected(window),
        ContentView::Label => label::recount_selected(window),
        _ => {}
    }
    true
}

