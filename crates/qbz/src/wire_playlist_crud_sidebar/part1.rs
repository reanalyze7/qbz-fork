use crate::*;

pub(crate) fn wire_playlist_crud_sidebar_part1(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {
    {
        let weak = window.as_weak();
        window.global::<DragActions>().on_move(move |gx, gy| {
            if let Some(w) = weak.upgrade() {
                let ds = w.global::<DragState>();
                ds.set_pointer_x(gx);
                ds.set_pointer_y(gy);
            }
        });
    }
}
