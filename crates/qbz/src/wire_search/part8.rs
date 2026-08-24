use crate::*;

pub(crate) fn wire_search_part8(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {

    // History navigation — back / forward / settings, all recorded by the
    // nav module so the [<] [>] pair and the mouse buttons stay in sync.
    {
        let weak = window.as_weak();
        window.global::<NavState>().on_request_settings(move || {
            nav::record(nav::NavEntry::Settings);
            if let Some(w) = weak.upgrade() {
                seed_blacklist_status(&w);
                w.global::<NavState>().set_view(ContentView::Settings);
                update_nav_flags(&w);
            }
        });
    }

    // Keyboard shortcuts (hotkeys): seed the cheatsheet/editor model + wire the
    // customize-editor capture callbacks. The global key dispatch itself lives
    // in `install_browser_mouse_nav`'s winit handler.
    keybindings::wire(&window);

    // "Open Qobuz Link" (Ctrl+L) — the cross-platform link resolver modal.
    {
        let weak = window.as_weak();
        window
            .global::<LinkResolverActions>()
            .on_url_changed(move |url| {
                if let Some(w) = weak.upgrade() {
                    w.global::<LinkResolverState>()
                        .set_platform(link_resolver::detect_platform(&url).into());
                }
            });
    }
}
