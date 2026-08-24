use crate::*;

pub(crate) fn wire_home_library_playback_part2(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {

    // Transport — wired through the NowPlayingState global callbacks.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<NowPlayingState>()
            .on_toggle_play(move || {
                let runtime = runtime.clone();
                let weak = weak.clone();
                let handle = handle.clone();
                handle.clone().spawn(async move {
                    playback::toggle_play_pause(runtime, weak, handle);
                });
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<NowPlayingState>().on_next(move || {
            let runtime = runtime.clone();
            let weak = weak.clone();
            let handle = handle.clone();
            handle.clone().spawn(async move {
                // NOTE: no cast-specific branch here. While casting, the local
                // next() flow runs — it moves the core cursor, refreshes the
                // now-playing card + queue, and calls play_audible, which casts
                // the new current track (the play_audible cast gate). Routing
                // next() through a cast-only path would advance the renderer but
                // leave the UI cursor stale (and then queue-click resolves
                // against the wrong index).
                playback::next(runtime, weak, handle);
            });
        });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<NowPlayingState>().on_previous(move || {
            let runtime = runtime.clone();
            let weak = weak.clone();
            let handle = handle.clone();
            handle.clone().spawn(async move {
                // See on_next: no cast branch — the local previous() flow keeps
                // the cursor + UI in sync and play_audible casts the new track.
                playback::previous(runtime, weak, handle);
            });
        });
    }
    {
        let runtime = app_runtime.clone();
        let handle = tokio_rt.handle().clone();
        window
            .global::<NowPlayingState>()
            .on_seek(move |fraction| {
                let runtime = runtime.clone();
                let handle = handle.clone();
                handle.clone().spawn(async move {
                    playback::seek(runtime, handle, fraction);
                });
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<NowPlayingState>()
            .on_set_volume(move |fraction| {
                let runtime = runtime.clone();
                let weak = weak.clone();
                let handle = handle.clone();
                handle.clone().spawn(async move {
                    playback::set_volume(runtime, weak, handle, fraction);
                });
            });
    }
}
