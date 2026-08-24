use crate::*;

pub(crate) fn wire_local_library_settings_part1(window: &AppWindow, _app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, _image_cache: &artwork::ImageCache, _settings_ctx: &Arc<settings::SettingsCtx>) {
    {
        let weak = window.as_weak();
        window
            .global::<LocalLibraryActions>()
            .on_open_settings(move || {
                // Management/maintenance/danger live under Settings > Local
                // Library — pre-select that sub-section (index 4). The panel's
                // `init` lazy-loads the folder list.
                nav::record(nav::NavEntry::Settings);
                if let Some(w) = weak.upgrade() {
                    w.global::<SettingsState>().set_section(4);
                    w.global::<NavState>().set_view(ContentView::Settings);
                    update_nav_flags(&w);
                }
            });
    }
    // Settings > Local Library — folder management + maintenance + danger.
    // (Scan callbacks scan-all/scan-folder/stop-scan are wired with Slice B.)
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LibraryManageActions>()
            .on_load(move || local_library_settings::load_folders(weak.clone(), handle.clone()));
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LibraryManageActions>()
            .on_add_folder(move || local_library_settings::add_folder(weak.clone(), handle.clone()));
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LibraryManageActions>()
            .on_remove_folders(move || {
                local_library_settings::remove_folders(weak.clone(), handle.clone())
            });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LibraryManageActions>()
            .on_remove_folder(move |id| {
                local_library_settings::remove_folder(weak.clone(), handle.clone(), id as i64)
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<LibraryManageActions>()
            .on_toggle_folder_select(move |id| {
                local_library_settings::toggle_select(weak.clone(), id as i64)
            });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LibraryManageActions>()
            .on_edit_folder(move |id| {
                local_library_settings::edit_folder(weak.clone(), handle.clone(), id as i64)
            });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LibraryManageActions>()
            .on_save_folder_settings(move |id, alias, enabled, is_network, fs_type, user_override| {
                local_library_settings::save_folder_settings(
                    weak.clone(),
                    handle.clone(),
                    id as i64,
                    alias.to_string(),
                    enabled,
                    is_network,
                    fs_type.to_string(),
                    user_override,
                )
            });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LibraryManageActions>()
            .on_change_folder_path(move |id| {
                local_library_settings::change_folder_path(weak.clone(), handle.clone(), id as i64)
            });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LibraryManageActions>()
            .on_cleanup_missing(move || {
                local_library_settings::cleanup_missing(weak.clone(), handle.clone())
            });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LibraryManageActions>()
            .on_clear_library(move || {
                local_library_settings::clear_library(weak.clone(), handle.clone())
            });
    }
}
