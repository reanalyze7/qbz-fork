use crate::*;

// --- Folder editor: open (new + edit) ------------------------------------
pub(crate) fn wire_pm_folder_open(window: &AppWindow) {
    {
        let weak = window.as_weak();
        window
            .global::<PlaylistManagerActions>()
            .on_new_folder(move || {
                if let Some(w) = weak.upgrade() {
                    let fes = w.global::<FolderEditState>();
                    fes.set_id("".into());
                    fes.set_name("".into());
                    fes.set_icon_preset("folder".into());
                    fes.set_icon_color("".into());
                    fes.set_is_hidden(false);
                    fes.set_custom_image_path("".into());
                    fes.set_open(true);
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<PlaylistManagerActions>()
            .on_edit_folder(move |id| {
                let Some(w) = weak.upgrade() else { return };
                open_folder_editor(&w, id);
            });
    }
}
