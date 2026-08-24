use crate::*;

// --- Folder editor: save + delete -----------------------------------------
pub(crate) fn wire_pm_folder_save_delete(
    window: &AppWindow,
    app_runtime: &Arc<AppRuntime<SlintAdapter>>,
    tokio_rt: &tokio::runtime::Runtime,
    image_cache: &artwork::ImageCache,
) {
    {
        // Save (create or update) the folder, then reload PM + sidebar.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<FolderEditActions>()
            .on_save(move || {
                let Some(w) = weak.upgrade() else { return };
                let fes = w.global::<FolderEditState>();
                let id = fes.get_id().to_string();
                let name = fes.get_name().to_string();
                if name.trim().is_empty() {
                    return;
                }
                let preset = fes.get_icon_preset().to_string();
                let color = fes.get_icon_color().to_string();
                let hidden = fes.get_is_hidden();
                let image_path = fes.get_custom_image_path().to_string();
                fes.set_open(false);
                let runtime = runtime.clone();
                let weak = weak.clone();
                let handle = handle.clone();
                let image_cache = image_cache.clone();
                handle.clone().spawn(async move {
                    let nm = name.trim().to_string();
                    tokio::task::spawn_blocking(move || {
                        if id.is_empty() {
                            folders::create_folder_full(&nm, &preset, &color);
                            // A custom image on a brand-new folder: set it
                            // in a follow-up update once we have the id.
                            // (Rare path; the create flow defaults to a
                            // preset icon — image edits use the edit path.)
                        } else {
                            let icon_type = if image_path.is_empty() { "preset" } else { "custom" };
                            let img = if image_path.is_empty() {
                                Some(None)
                            } else {
                                Some(Some(image_path.as_str()))
                            };
                            folders::update_folder_full(
                                &id, &nm, icon_type, &preset, &color, img, hidden,
                            );
                        }
                    })
                    .await
                    .ok();
                    // Reload the manager data + sidebar.
                    let data = playlist_manager::load(&runtime).await;
                    let weak2 = weak.clone();
                    let r2 = runtime.clone();
                    let h2 = handle.clone();
                    let ic = image_cache.clone();
                    let _ = weak.upgrade_in_event_loop(move |w| {
                        playlist_manager::apply(&w, data);
                        refresh_pm_covers(&w);
                        load_sidebar_playlists(r2, weak2, &h2);
                        let _ = ic;
                    });
                });
            });
    }
    {
        // Delete the folder (Tauri ask() confirm), then reload.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<FolderEditActions>()
            .on_delete(move || {
                let Some(w) = weak.upgrade() else { return };
                let id = w.global::<FolderEditState>().get_id().to_string();
                let name = w.global::<FolderEditState>().get_name().to_string();
                if id.is_empty() {
                    return;
                }
                let runtime = runtime.clone();
                let weak = weak.clone();
                let handle = handle.clone();
                handle.clone().spawn(async move {
                    let confirmed = rfd::AsyncMessageDialog::new()
                        .set_title("Delete folder")
                        .set_description(format!(
                            "Delete the folder \u{201c}{name}\u{201d}? Its playlists move back to the root."
                        ))
                        .set_buttons(rfd::MessageButtons::YesNo)
                        .show()
                        .await;
                    if confirmed != rfd::MessageDialogResult::Yes {
                        return;
                    }
                    let fid = id.clone();
                    tokio::task::spawn_blocking(move || folders::delete_folder(&fid))
                        .await
                        .ok();
                    let _ = weak.upgrade_in_event_loop(|w| {
                        w.global::<FolderEditState>().set_open(false);
                    });
                    let data = playlist_manager::load(&runtime).await;
                    let weak2 = weak.clone();
                    let r2 = runtime.clone();
                    let h2 = handle.clone();
                    let _ = weak.upgrade_in_event_loop(move |w| {
                        playlist_manager::apply(&w, data);
                        refresh_pm_covers(&w);
                        load_sidebar_playlists(r2, weak2, &h2);
                    });
                });
            });
    }
}
