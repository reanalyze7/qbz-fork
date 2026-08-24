use crate::*;

// SPLIT-EXCEPTION (crates/qbz/src/main.rs refactor): this fn wraps ONE
// original fn main() statement (a single Slint callback registration or
// startup step) too internally sequential/closure-heavy to decompose
// further without a compiler in the loop (no `cargo check` is permitted
// for this refactor). Left whole, over the 130-line rule, as the
// documented rare exception it allows for.
pub(crate) fn wire_discover_offline_manager_part3(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {

    // Offline Cache Manager actions.
    {
        let runtime = app_runtime.clone();
        let handle = tokio_rt.handle().clone();
    {
        let weak = window.as_weak();
        let handle = handle.clone();
        window.global::<OfflineManagerActions>().on_open(move || {
            nav::record(nav::NavEntry::OfflineManager);
            if let Some(w) = weak.upgrade() {
                w.global::<NavState>().set_view(ContentView::OfflineManager);
                update_nav_flags(&w);
            }
            offline_manager::load(weak.clone(), handle.clone());
        });
    }
    {
        let weak = window.as_weak();
        let handle = handle.clone();
        window.global::<OfflineManagerActions>().on_refresh(move || {
            offline_manager::load(weak.clone(), handle.clone());
        });
    }
    {
        let weak = window.as_weak();
        let handle = handle.clone();
        window
            .global::<OfflineManagerActions>()
            .on_select_artist(move |name| {
                offline_manager::select_artist(weak.clone(), handle.clone(), name.to_string());
            });
    }
    {
        let weak = window.as_weak();
        let handle = handle.clone();
        window
            .global::<OfflineManagerActions>()
            .on_set_sort(move |i| {
                offline_manager::set_sort(weak.clone(), handle.clone(), i);
            });
    }
    {
        let weak = window.as_weak();
        let handle = handle.clone();
        window
            .global::<OfflineManagerActions>()
            .on_toggle_failed(move || {
                offline_manager::toggle_failed(weak.clone(), handle.clone());
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<OfflineManagerActions>()
            .on_toggle_select(move |id| {
                if let Some(w) = weak.upgrade() {
                    offline_manager::toggle_select(&w, &id);
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<OfflineManagerActions>()
            .on_select_all(move || {
                if let Some(w) = weak.upgrade() {
                    offline_manager::set_all_selected(&w, true);
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<OfflineManagerActions>()
            .on_clear_selection(move || {
                if let Some(w) = weak.upgrade() {
                    offline_manager::set_all_selected(&w, false);
                }
            });
    }
    {
        let weak = window.as_weak();
        let runtime = runtime.clone();
        let handle = handle.clone();
        window
            .global::<OfflineManagerActions>()
            .on_bulk_redownload(move || {
                if let Some(w) = weak.upgrade() {
                    for id in offline_manager::selected_track_ids(&w) {
                        offline_cache::redownload_track(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            id,
                        );
                    }
                }
            });
    }
    {
        let weak = window.as_weak();
        let runtime = runtime.clone();
        let handle = handle.clone();
        window
            .global::<OfflineManagerActions>()
            .on_bulk_remove(move || {
                if let Some(w) = weak.upgrade() {
                    for id in offline_manager::selected_track_ids(&w) {
                        offline_cache::remove_cached(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            id,
                        );
                    }
                }
            });
    }
    {
        let weak = window.as_weak();
        let runtime = runtime.clone();
        let handle = handle.clone();
        window
            .global::<OfflineManagerActions>()
            .on_remove_track(move |id| {
                if let Ok(tid) = id.parse::<u64>() {
                    offline_cache::remove_cached(runtime.clone(), weak.clone(), handle.clone(), tid);
                }
            });
    }
    {
        let weak = window.as_weak();
        let handle = handle.clone();
        window
            .global::<OfflineManagerActions>()
            .on_remove_album(move |aid| {
                offline_cache::remove_album(weak.clone(), handle.clone(), aid.to_string());
            });
    }
    {
        let weak = window.as_weak();
        let runtime = runtime.clone();
        let handle = handle.clone();
        window
            .global::<OfflineManagerActions>()
            .on_redownload_track(move |id| {
                if let Ok(tid) = id.parse::<u64>() {
                    offline_cache::redownload_track(
                        runtime.clone(),
                        weak.clone(),
                        handle.clone(),
                        tid,
                    );
                }
            });
    }
    {
        let weak = window.as_weak();
        let runtime = runtime.clone();
        let handle = handle.clone();
        window
            .global::<OfflineManagerActions>()
            .on_redownload_album(move |aid| {
                offline_cache::redownload_album(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    aid.to_string(),
                    false,
                );
            });
    }
    {
        let weak = window.as_weak();
        let runtime = runtime.clone();
        let handle = handle.clone();
        window
            .global::<OfflineManagerActions>()
            .on_redownload_failed(move |aid| {
                offline_cache::redownload_album(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    aid.to_string(),
                    true,
                );
            });
    }
    {
        let weak = window.as_weak();
        let handle = handle.clone();
        window
            .global::<OfflineManagerActions>()
            .on_set_limit(move |gb| {
                offline_manager::set_limit(weak.clone(), handle.clone(), gb);
            });
    }
    {
        let weak = window.as_weak();
        let handle = handle.clone();
        window.global::<OfflineManagerActions>().on_clear_all(move || {
            offline_cache::clear_all(weak.clone(), handle.clone());
        });
    }
    {
        let handle = handle.clone();
        window
            .global::<OfflineManagerActions>()
            .on_open_folder(move || {
                offline_cache::open_folder(handle.clone());
            });
    }
    {
        let weak = window.as_weak();
        let runtime = runtime.clone();
        let handle = handle.clone();
        window
            .global::<OfflineManagerActions>()
            .on_play_track(move |id| {
                if let Ok(tid) = id.parse::<u64>() {
                    playback::play_track_now(runtime.clone(), weak.clone(), handle.clone(), tid);
                }
            });
    }
    }
}
