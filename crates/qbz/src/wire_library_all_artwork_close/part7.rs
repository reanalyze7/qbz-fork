use crate::*;

pub(crate) fn wire_library_all_artwork_close_part7(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {

    // Intercept the window-manager close (native titlebar X / compositor
    // close). Mirrors the custom titlebar: hide to tray when close-to-tray is
    // on + the tray is live, otherwise quit. Required because the loop runs
    // with quit_on_last_window_closed = false (so a tray-hide keeps the app
    // alive) — without this, the native close would leave a headless process.
    window.window().on_close_requested(move || {
        let settings = tray_settings::get();
        if settings.close_to_tray && tray::handle().is_some() {
            // Slint performs the hide (destroys the surface) for HideWindow;
            // we only sync the shown flag so the next tray toggle shows it.
            log::info!("[qbz-slint] close-to-tray (WM close): hiding to tray");
            // Flush the session even when only hiding — the process may be
            // killed from the tray / shell without a real quit afterwards.
            session_persist::save_on_exit();
            tray::set_window_shown(false);
            // macOS: drop the Dock icon if the user opted in (no-op elsewhere).
            if settings.mac_hide_dock {
                tray::set_mac_dock_hidden(true);
            }
            slint::CloseRequestResponse::HideWindow
        } else {
            log::info!("[qbz-slint] WM close requested: quitting");
            // Flush the final session snapshot before quitting.
            session_persist::save_on_exit();
            let _ = slint::quit_event_loop();
            slint::CloseRequestResponse::HideWindow
        }
    });

    window.on_open_tos(|| {
        dispatch(AppCommand::OpenTermsOfService);
        if let Err(e) = open::that(QOBUZ_TOS_URL) {
            log::error!("[qbz-slint] failed to open Terms of Service: {e}");
        }
    });

    log::info!("[qbz-slint] window ready");
    // NOT `window.run()`: that quits the event loop when the last window
    // closes, which would kill the app the moment the window hides to tray.
    // `run_event_loop_until_quit()` keeps the loop alive until an explicit
    // `quit_event_loop()` (custom titlebar / WM close when not close-to-tray /
    // tray Quit), so hide-to-tray works.
    window.show()?;
    // macOS custom chrome: centre the native traffic lights in the 42px
    // header — AppKit parks them at the stock ~28pt titlebar height, visibly
    // above the header controls. Queued so it runs after the event loop has
    // processed the show (the NSWindow/handle only exists then).
    #[cfg(target_os = "macos")]
    {
        let weak = window.as_weak();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(w) = weak.upgrade() {
                macos_chrome::center_traffic_lights(w.window());
            }
        });
    }
    slint::run_event_loop_until_quit()?;
    // Reaching here = a CLEAN exit (tray Quit calls quit_event_loop directly,
    // with no window input event) — without this, launch-then-quit-from-tray
    // inside the 30s fallback would leave the sentinel armed and falsely
    // walk the renderer ladder on the next start.
    disarm_renderer_sentinel_on_liveness("clean exit");
    // Single choke point for ALL quit paths (custom-titlebar close, WM close,
    // tray Quit): release anything QBZ parked on the audio graph before the
    // process exits. Quitting mid-playback never runs the audio thread's Stop
    // handler, so a forced PipeWire clock (DAC passthrough) outlived the app
    // and pinned every other program to the last track's sample rate until
    // PipeWire restarted (#521). Both calls are self-gating no-ops when QBZ
    // didn't set anything — same pair the Stop/ReleaseDevice handlers use.
    #[cfg(target_os = "linux")]
    {
        qbz_audio::alsa_backend::resume_suspended_sink();
        qbz_audio::pipewire_backend::PipeWireBackend::reset_pipewire_clock();
    }
}
