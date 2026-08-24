use crate::*;

pub(crate) fn wire_home_library_playback_part3(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, _image_cache: &artwork::ImageCache, _settings_ctx: &Arc<settings::SettingsCtx>) {
    // Persist sidebar state / section-nav placement / volume (drag-end only)
    // to ui_prefs. These callbacks just touch the prefs file — no runtime.
    {
        let shell = window.global::<ShellState>();
        shell.on_persist_sidebar_state(|state| {
            let mut prefs = crate::ui_prefs::load();
            prefs.sidebar_state = state;
            crate::ui_prefs::save(&prefs);
        });
        shell.on_persist_nav(|enabled| {
            let mut prefs = crate::ui_prefs::load();
            prefs.nav_in_sidebar = enabled;
            crate::ui_prefs::save(&prefs);
        });
        shell.on_persist_nav_compact(|enabled| {
            let mut prefs = crate::ui_prefs::load();
            prefs.nav_header_compact = enabled;
            crate::ui_prefs::save(&prefs);
        });
        window.global::<NowPlayingState>().on_persist_volume(|fraction| {
            let mut prefs = crate::ui_prefs::load();
            prefs.volume = fraction.clamp(0.0, 1.0);
            crate::ui_prefs::save(&prefs);
        });
        // Remember the last SAFE top-level view for "where you left off".
        let weak = window.as_weak();
        shell.on_persist_view(move || {
            let Some(w) = weak.upgrade() else { return };
            let mut prefs = crate::ui_prefs::load();
            let mut dirty = false;
            // Legacy top-level key (offline-restore fallback).
            if let Some(key) = safe_view_key(w.global::<NavState>().get_view()) {
                if prefs.last_view != key {
                    prefs.last_view = key.to_string();
                    dirty = true;
                }
            }
            // Full entry for exact restore. Skip transient/config destinations
            // (a relaunch into the live-search results page or Settings is not
            // "where you left off"); those keep the prior last_nav.
            if let Some(entry) = nav::current() {
                let persistable =
                    !matches!(entry, nav::NavEntry::Search(_) | nav::NavEntry::Settings);
                if persistable {
                    if let Ok(json) = serde_json::to_string(&entry) {
                        if prefs.last_nav.as_deref() != Some(json.as_str()) {
                            prefs.last_nav = Some(json);
                            dirty = true;
                        }
                    }
                }
            }
            if dirty {
                crate::ui_prefs::save(&prefs);
            }
        });
    }

    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<NowPlayingState>()
            .on_toggle_mute(move || {
                let runtime = runtime.clone();
                let weak = weak.clone();
                let handle = handle.clone();
                handle.clone().spawn(async move {
                    playback::toggle_mute(runtime, weak, handle);
                });
            });
    }

    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<NowPlayingState>()
            .on_toggle_shuffle(move || {
                let runtime = runtime.clone();
                let weak = weak.clone();
                let handle = handle.clone();
                handle.clone().spawn(async move {
                    playback::toggle_shuffle(runtime, weak, handle);
                });
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<NowPlayingState>()
            .on_cycle_repeat(move || {
                let runtime = runtime.clone();
                let weak = weak.clone();
                let handle = handle.clone();
                handle.clone().spawn(async move {
                    playback::cycle_repeat(runtime, weak, handle);
                });
            });
    }
}
