use crate::*;

// --- Folder editor: field changes -----------------------------------------
pub(crate) fn wire_pm_folder_fields(window: &AppWindow, tokio_rt: &tokio::runtime::Runtime) {
    {
        let weak = window.as_weak();
        window
            .global::<FolderEditActions>()
            .on_select_preset(move |id| {
                if let Some(w) = weak.upgrade() {
                    let fes = w.global::<FolderEditState>();
                    fes.set_icon_preset(id);
                    // Choosing a preset clears the custom image.
                    fes.set_custom_image_path("".into());
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<FolderEditActions>()
            .on_select_color(move |hex| {
                if let Some(w) = weak.upgrade() {
                    w.global::<FolderEditState>().set_icon_color(hex);
                }
            });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<FolderEditActions>()
            .on_pick_image(move || {
                let weak = weak.clone();
                handle.spawn(async move {
                    let Some(file) = rfd::AsyncFileDialog::new()
                        .add_filter("Images", &["png", "jpg", "jpeg", "webp", "gif"])
                        .pick_file()
                        .await
                    else {
                        return;
                    };
                    let path = file.path().to_string_lossy().to_string();
                    let path2 = path.clone();
                    let _ = weak.upgrade_in_event_loop(move |w| {
                        w.global::<FolderEditState>().set_custom_image_path(path2.into());
                        playlist_manager::load_editor_custom_image(w.as_weak(), path);
                    });
                });
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<FolderEditActions>()
            .on_clear_image(move || {
                if let Some(w) = weak.upgrade() {
                    w.global::<FolderEditState>().set_custom_image_path("".into());
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<FolderEditActions>()
            .on_toggle_hidden(move || {
                if let Some(w) = weak.upgrade() {
                    let fes = w.global::<FolderEditState>();
                    fes.set_is_hidden(!fes.get_is_hidden());
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<FolderEditActions>()
            .on_close(move || {
                if let Some(w) = weak.upgrade() {
                    w.global::<FolderEditState>().set_open(false);
                }
            });
    }
}
