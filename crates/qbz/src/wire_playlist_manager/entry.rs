use crate::*;

/// Wire all Playlist Manager + folder-editor callbacks. Mirrors the
/// favorites + sidebar wiring: optimistic local mutations (rebuild from
/// cache) plus a backend write on a blocking thread.
///
/// Split (2nd pass) into one `wire_pm_*` helper per callback group, called
/// here in the exact original registration order — each group is an
/// independent `{ ... window.global::<X>().on_y(...) }` block with no
/// shared control-flow across groups, so the split preserves behavior
/// exactly.
pub(crate) fn wire_playlist_manager(
    window: &AppWindow,
    app_runtime: &Arc<AppRuntime<SlintAdapter>>,
    tokio_rt: &tokio::runtime::Runtime,
    image_cache: &artwork::ImageCache,
) {
    // The folder-editor preset + color grids (built once, never change).
    {
        let (presets, swatches) = folder_editor_presets();
        let fes = window.global::<FolderEditState>();
        fes.set_icon_presets(slint::ModelRc::new(slint::VecModel::from(presets)));
        fes.set_color_swatches(slint::ModelRc::new(slint::VecModel::from(swatches)));
    }

    wire_pm_open(window, app_runtime, tokio_rt, image_cache);
    wire_pm_toolbar(window);
    wire_pm_per_card_flags(window, app_runtime, tokio_rt);
    wire_pm_per_card_edit(window, tokio_rt);
    wire_pm_reorder(window, app_runtime, tokio_rt);
    wire_pm_folder_open(window);
    wire_pm_folder_fields(window, tokio_rt);
    wire_pm_folder_save_delete(window, app_runtime, tokio_rt, image_cache);
}
