use crate::*;

pub(crate) fn wire_link_and_import_part3(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, _image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {

    // Settings — the output-device refresh/release button: free a device QBZ
    // holds exclusively (ALSA Direct) and re-enumerate, so a freed or
    // hot-plugged DAC reappears without an app restart.
    {
        let runtime = app_runtime.clone();
        let settings_ctx = settings_ctx.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.on_settings_release_device(move || {
            let runtime = runtime.clone();
            let settings_ctx = settings_ctx.clone();
            let weak = weak.clone();
            handle.spawn(async move {
                settings::handle_release_device(settings_ctx, runtime, weak).await;
            });
        });
    }

    // Settings > Developer — "Export settings…" modal confirm: build the
    // settings bundle via the shared engine, open a native save dialog, write
    // it 0600, and toast the import command (04 §4.2). No new export logic.
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<SettingsExportActions>().on_confirm(move || {
            settings::export_settings(weak.clone(), handle.clone());
        });
    }

    // Settings > Offline MODE — re-seed the toggle states on panel mount
    // (the panel's init fires load), and the status row's "Check now"
    // connectivity re-probe. The toggles themselves persist through the
    // generic settings-bool path above.
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<OfflineModeActions>().on_load(move || {
            offline_mode::seed_settings(weak.clone(), handle.clone());
        });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<OfflineModeActions>().on_check_now(move || {
            offline_mode::check_now(weak.clone(), handle.clone());
        });
    }
    // The header badge flyout's quick offline toggle — same persistence +
    // #279 snapshot path as the Settings "Enable Offline Mode" toggle.
    {
        let runtime = app_runtime.clone();
        let settings_ctx = settings_ctx.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<OfflineModeActions>()
            .on_set_offline(move |value| {
                let runtime = runtime.clone();
                let settings_ctx = settings_ctx.clone();
                let weak = weak.clone();
                handle.spawn(async move {
                    settings::handle_bool(
                        settings_ctx,
                        runtime,
                        weak,
                        "offline-mode-enabled".to_string(),
                        value,
                    )
                    .await;
                });
            });
    }

    // B9 — offline Favorites "playable favorites" rail: rebuild on every
    // mount of the Favorites offline placeholder (the rail's init fires
    // load), play the rail from the clicked row.
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<OfflineFavoritesActions>().on_load(move || {
            offline_favorites::load(weak.clone(), handle.clone());
        });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<OfflineFavoritesActions>().on_play(move |id| {
            offline_favorites::play(
                runtime.clone(),
                weak.clone(),
                handle.clone(),
                id.to_string(),
            );
        });
    }
}
