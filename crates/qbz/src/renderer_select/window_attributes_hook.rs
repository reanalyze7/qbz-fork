
// Make ONLY the miniplayer window borderless at CREATION (the flag is true
// solely while MiniPlayerWindow::new() runs). Decorations cannot be
// reliably removed post-creation on Wayland/KDE (server-side decorations
// are negotiated when the surface is created), so the AppWindow keeps its
// system titlebar while the mini never has one. Split out of
// `select_slint_backend` (part6.rs) to stay under the 130-line file cap —
// a plain fn item (no captures) coerces to the `Fn` bound
// `with_winit_window_attributes_hook` expects, same as the original
// closure.
pub(crate) fn window_attributes_hook(
    attributes: i_slint_backend_winit::winit::window::WindowAttributes,
) -> i_slint_backend_winit::winit::window::WindowAttributes {
        // The miniplayer window is gone; the main window keeps its system chrome.
        let creating_mini = false;
        // Wayland app_id / X11 WM_CLASS: without an explicit name winit sends
        // no xdg_toplevel.set_app_id at all (and derives WM_CLASS from the
        // binary name), so the compositor cannot match the window to
        // io.github.reanalyze7.qoqobuz.desktop — blank dock icon, no running indicator,
        // and clicking the pin spawns a second instance (#544). Set on BOTH
        // windows so the miniplayer groups under the same icon.
        #[cfg(all(unix, not(target_os = "macos")))]
        let attributes = {
            use i_slint_backend_winit::winit::platform::wayland::WindowAttributesExtWayland;
            use i_slint_backend_winit::winit::platform::x11::WindowAttributesExtX11;
            // Both traits expose `with_name`; UFCS picks each apart.
            let attributes = WindowAttributesExtWayland::with_name(
                attributes,
                "io.github.reanalyze7.qoqobuz",
                "io.github.reanalyze7.qoqobuz",
            );
            WindowAttributesExtX11::with_name(attributes, "io.github.reanalyze7.qoqobuz", "io.github.reanalyze7.qoqobuz")
        };
        // Per-window swapchain alpha (vendored femtovg-wgpu patch): this hook runs
        // on the event loop thread right before the window ADAPTER — and therefore
        // its renderer backend — is created, and the backend CAPTURES the flag at
        // construction (surface (re)creation happens later and repeats on every
        // Wayland re-show, so a live read there would leak this latched value
        // across windows). Net effect: only the miniplayer gets a transparent
        // (blended) swapchain, for its whole lifetime; the main window keeps an
        // Opaque one, sparing the compositor a full-window alpha blend every frame.
        #[cfg(not(target_os = "macos"))]
        i_slint_renderer_femtovg::wgpu::set_surface_prefers_transparent(creating_mini);
        // macOS custom chrome (owner decision 2026-07-03, default ON there):
        // keep the native decorations but make the title bar transparent and
        // extend the content underneath — the native traffic lights float over
        // the app's own header (which reserves a left inset for them). This is
        // the macOS analog of Linux's `no-frame`; we never draw Mac controls.
        // Same restart-to-apply semantics: attributes are fixed at creation.
        #[cfg(target_os = "macos")]
        let attributes = if !creating_mini && !crate::ui_prefs::load().use_system_title_bar {
            use i_slint_backend_winit::winit::platform::macos::WindowAttributesExtMacOS;
            attributes
                .with_titlebar_transparent(true)
                .with_title_hidden(true)
                .with_fullsize_content_view(true)
        } else {
            attributes
        };
        if creating_mini {
            attributes.with_decorations(false)
        } else {
            attributes
        }
}
