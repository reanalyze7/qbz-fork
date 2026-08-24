use crate::*;

// --- Toolbar -------------------------------------------------------------
pub(crate) fn wire_pm_toolbar(window: &AppWindow) {
    {
        let weak = window.as_weak();
        window
            .global::<PlaylistManagerActions>()
            .on_search_changed(move |query| {
                if let Some(w) = weak.upgrade() {
                    w.global::<PlaylistManagerState>().set_search_query(query);
                    playlist_manager::rebuild(&w);
                    refresh_pm_covers(&w);
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<PlaylistManagerActions>()
            .on_search_folders(move |query| {
                if let Some(w) = weak.upgrade() {
                    playlist_manager::search_menu_folders(&w, &query);
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<PlaylistManagerActions>()
            .on_set_filter(move |value| {
                if let Some(w) = weak.upgrade() {
                    w.global::<PlaylistManagerState>().set_filter(value);
                    playlist_manager::rebuild(&w);
                    refresh_pm_covers(&w);
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<PlaylistManagerActions>()
            .on_set_sort(move |value| {
                if let Some(w) = weak.upgrade() {
                    w.global::<PlaylistManagerState>().set_sort(value);
                    playlist_manager::rebuild(&w);
                    refresh_pm_covers(&w);
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<PlaylistManagerActions>()
            .on_set_view_mode(move |value| {
                if let Some(w) = weak.upgrade() {
                    w.global::<PlaylistManagerState>().set_view_mode(value);
                    playlist_manager::rebuild(&w);
                    refresh_pm_covers(&w);
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<PlaylistManagerActions>()
            .on_toggle_folder_mode(move || {
                if let Some(w) = weak.upgrade() {
                    let st = w.global::<PlaylistManagerState>();
                    let next = !st.get_folder_mode();
                    st.set_folder_mode(next);
                    // Leaving folder mode while in tree falls back to grid.
                    if !next && st.get_view_mode() == "tree" {
                        st.set_view_mode("grid".into());
                    }
                    playlist_manager::rebuild(&w);
                    refresh_pm_covers(&w);
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<PlaylistManagerActions>()
            .on_toggle_folders_collapsed(move || {
                if let Some(w) = weak.upgrade() {
                    let st = w.global::<PlaylistManagerState>();
                    st.set_folders_collapsed(!st.get_folders_collapsed());
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<PlaylistManagerActions>()
            .on_toggle_tree_folder(move |id| {
                if let Some(w) = weak.upgrade() {
                    playlist_manager::toggle_tree_folder(&w, id.as_str());
                    refresh_pm_covers(&w);
                }
            });
    }
}
