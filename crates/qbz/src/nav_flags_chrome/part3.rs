use crate::*;

/// Escape: leave multi-select mode on the active track surface (clears the
/// selection AND turns the mode off, via the surface's `set_multi_select`,
/// which also drops the Shift-range anchor). Returns true iff a surface was in
/// select mode (so Escape is consumed before the queue/other dismissables).
pub(crate) fn exit_active_multi_select(window: &AppWindow) -> bool {
    let view = window.global::<NavState>().get_view();
    match view {
        ContentView::Album if window.global::<AlbumState>().get_multi_select() => {
            album::set_multi_select(window, false);
            true
        }
        ContentView::Artist
            if window.global::<ArtistState>().get_top_tracks_multi_select() =>
        {
            artist::set_multi_select(window, false);
            true
        }
        ContentView::Playlist if window.global::<PlaylistState>().get_multi_select_mode() => {
            playlist::set_multi_select(window, false);
            true
        }
        ContentView::Favorites
            if window.global::<FavoritesState>().get_tracks_multi_select() =>
        {
            favorites::set_multi_select(window, false);
            true
        }
        ContentView::LocalLibrary
            if window.global::<LocalLibraryState>().get_albums_multi_select() =>
        {
            local_library::set_albums_multi_select(window, false);
            true
        }
        ContentView::LocalLibrary
            if window.global::<LocalLibraryState>().get_tracks_multi_select() =>
        {
            local_library::set_tracks_multi_select(window, false);
            true
        }
        ContentView::Mix if window.global::<MixState>().get_multi_select() => {
            mix::set_multi_select(window, false);
            true
        }
        ContentView::Label if window.global::<LabelState>().get_multi_select() => {
            label::set_multi_select(window, false);
            true
        }
        // Offline Manager has no mode to leave (always-on selection); Escape
        // clears the selection when something is selected.
        ContentView::OfflineManager
            if window.global::<OfflineManagerState>().get_selected_count() > 0 =>
        {
            offline_manager::set_all_selected(window, false);
            crate::selection::clear_anchor();
            true
        }
        _ => false,
    }
}

