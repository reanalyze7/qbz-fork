use crate::*;

pub(crate) fn wire_home_library_playback_part4(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {

    // Queue sidebar — build the controller and wire every callback.
    {
        let controller = queue::QueueController::new(
            app_runtime.clone(),
            window.as_weak(),
            tokio_rt.handle().clone(),
            settings_ctx.playback_prefs(),
        );
        // Publish it so the playback paths refresh the sidebar after every
        // queue mutation (play / skip / auto-advance / enqueue).
        playback::set_queue_controller(controller.clone());

        let qs = window.global::<QueueState>();
        {
            let c = controller.clone();
            qs.on_play_upcoming(move |index| c.play_upcoming(index.max(0) as usize));
        }
        {
            let c = controller.clone();
            qs.on_play_coverflow_upcoming(move |index| {
                c.play_coverflow_upcoming(index.max(0) as usize)
            });
        }
        {
            let c = controller.clone();
            qs.on_play_history(move |index| c.play_history(index.max(0) as usize));
        }
        {
            let c = controller.clone();
            qs.on_remove_upcoming(move |index| c.remove_upcoming(index.max(0) as usize));
        }
        {
            let c = controller.clone();
            qs.on_remove_all_after(move |index| c.remove_all_after(index.max(0) as usize));
        }
        {
            let c = controller.clone();
            qs.on_add_to_playlist(move |index| c.add_to_playlist(index.max(0) as usize));
        }
        {
            let c = controller.clone();
            qs.on_reorder(move |from, to| {
                c.reorder(from.max(0) as usize, to.max(0) as usize);
            });
        }
        {
            let c = controller.clone();
            qs.on_clear_queue(move || c.clear());
        }
        {
            let c = controller.clone();
            qs.on_toggle_now_playing_favorite(move || c.toggle_favorite());
        }
        {
            let c = controller.clone();
            qs.on_save_as_playlist(move || c.save_as_playlist());
        }
        {
            let c = controller.clone();
            qs.on_toggle_infinite_play(move || c.toggle_infinite_play());
        }
        {
            let c = controller.clone();
            qs.on_toggle_stop_after(move |id| c.toggle_stop_after(id.to_string()));
        }
        // Sleep timer (queue footer): a Rust-owned tokio task drives the countdown
        // and pauses playback at the deadline.
        {
            let runtime = app_runtime.clone();
            let weak = window.as_weak();
            let handle = tokio_rt.handle().clone();
            window
                .global::<SleepTimerActions>()
                .on_set(move |minutes| {
                    sleep_timer::set(runtime.clone(), weak.clone(), handle.clone(), minutes)
                });
        }
        {
            let weak = window.as_weak();
            window
                .global::<SleepTimerActions>()
                .on_cancel(move || sleep_timer::cancel(weak.clone()));
        }
        // Developer panel: in-app log viewer + the full diagnostics panel.
        log_viewer::install(&window, app_runtime.clone(), tokio_rt.handle().clone());
        diagnostics::install(&window, app_runtime.clone(), tokio_rt.handle().clone());
        // Report-an-issue: "Create issue report" opens the GitHub new-issue page.
        window.global::<ReportIssueActions>().on_create_issue(|| {
            let url = "https://github.com/vicrodh/qbz/issues/new?template=bug_report.yml";
            if let Err(e) = open::that(url) {
                log::warn!("[qbz-slint] open GitHub issues failed: {e}");
            }
        });
        // About QBZ (static seed + open-url) and What's New (fetch on open).
        about::install(&window, tokio_rt.handle().clone());
        whats_new::install(&window, tokio_rt.handle().clone());
        {
            let c = controller.clone();
            let weak = window.as_weak();
            qs.on_search_changed(move || {
                let query = weak
                    .upgrade()
                    .map(|w| w.global::<QueueState>().get_search_query().to_string())
                    .unwrap_or_default();
                c.search_changed(query);
            });
        }
        {
            let c = controller.clone();
            qs.on_prev_page(move || c.prev_page());
        }
        {
            let c = controller.clone();
            qs.on_next_page(move || c.next_page());
        }
        {
            let c = controller.clone();
            qs.on_set_tab(move |tab| c.set_tab(tab));
        }
        {
            let c = controller.clone();
            // On open, also re-pull favorites so the heart is accurate.
            qs.on_panel_opened(move || c.refresh_with_favorites());
        }
    }
}
