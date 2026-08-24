use crate::*;

pub(crate) fn wire_local_library_settings_part13(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {

    // ---- Ephemeral folder actions ----
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LocalLibraryActions>()
            .on_ephemeral_open(move || {
                local_library::open_ephemeral(runtime.clone(), weak.clone(), handle.clone());
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LocalLibraryActions>()
            .on_ephemeral_play_all(move || {
                playback::ephemeral_play_or_prompt(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    "all".to_string(),
                    String::new(),
                );
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LocalLibraryActions>()
            .on_ephemeral_play_track(move |id| {
                playback::ephemeral_play_or_prompt(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    "track".to_string(),
                    id.to_string(),
                );
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LocalLibraryActions>()
            .on_ephemeral_play_album(move |key| {
                playback::ephemeral_play_or_prompt(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    "album".to_string(),
                    key.to_string(),
                );
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LocalLibraryActions>()
            .on_ephemeral_clear(move || {
                let runtime = runtime.clone();
                let weak = weak.clone();
                handle.spawn(async move {
                    // Stop a playing ephemeral track before dropping the session
                    // so its (about-to-be-reused) id can't false-highlight rows.
                    playback::wipe_ephemeral_if_playing(&runtime, &weak).await;
                    let _ = weak.upgrade_in_event_loop(|w| {
                        local_library::clear_ephemeral(&w);
                    });
                });
            });
    }
    // Ephemeral "already playing" choice modal — clear-and-play vs add-to-queue.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<EphemeralPlayChoiceActions>()
            .on_replace(move || {
                if let Some(w) = weak.upgrade() {
                    let s = w.global::<EphemeralPlayChoiceState>();
                    let kind = s.get_intent_kind().to_string();
                    let arg = s.get_intent_arg().to_string();
                    s.set_open(false);
                    playback::ephemeral_play(
                        runtime.clone(),
                        weak.clone(),
                        handle.clone(),
                        kind,
                        arg,
                    );
                }
            });
    }
}
