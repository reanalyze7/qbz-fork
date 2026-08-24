use crate::*;

// SPLIT-EXCEPTION (crates/qbz/src/main.rs refactor): this fn wraps ONE
// original fn main() statement (a single Slint callback registration or
// startup step) too internally sequential/closure-heavy to decompose
// further without a compiler in the loop (no `cargo check` is permitted
// for this refactor). Left whole, over the 130-line rule, as the
// documented rare exception it allows for.
pub(crate) fn wire_queue_and_cards_part10(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {

    // Track Info + Album Info modal actions (close / tab / navigation / play).
    // Navigation reuses the same handlers the rest of the app uses (open-artist
    // callback, network-sidebar musician resolve, navigate_label).
    {
        let runtime = app_runtime.clone();
        // -- Track Info --
        let weak = window.as_weak();
        window
            .global::<TrackInfoActions>()
            .on_close(move || {
                if let Some(w) = weak.upgrade() {
                    w.global::<TrackInfoState>().set_open(false);
                }
            });
        let weak = window.as_weak();
        window
            .global::<TrackInfoActions>()
            .on_open_artist(move |artist_id| {
                if let Some(w) = weak.upgrade() {
                    w.global::<TrackInfoState>().set_open(false);
                    w.invoke_open_artist(artist_id);
                }
            });
        let weak = window.as_weak();
        let runtime_l = runtime.clone();
        let handle_l = tokio_rt.handle().clone();
        let image_cache_l = image_cache.clone();
        window
            .global::<TrackInfoActions>()
            .on_open_label(move |label_id| {
                if let Some(w) = weak.upgrade() {
                    let name = w.global::<TrackInfoState>().get_label().to_string();
                    w.global::<TrackInfoState>().set_open(false);
                    if let Ok(id) = label_id.parse::<u64>() {
                        navigate_label(
                            runtime_l.clone(),
                            w.as_weak(),
                            &handle_l,
                            image_cache_l.clone(),
                            id,
                            name,
                        );
                    }
                }
            });
        let weak = window.as_weak();
        window
            .global::<TrackInfoActions>()
            .on_open_musician(move |name, role| {
                if let Some(w) = weak.upgrade() {
                    w.global::<TrackInfoState>().set_open(false);
                    w.global::<NetworkSidebarActions>()
                        .invoke_musician_clicked(name, role);
                }
            });
        // Immersive split Track Info panel: populate TrackInfoState for the
        // given track WITHOUT opening the floating modal (open stays false).
        let weak = window.as_weak();
        let runtime_l = runtime.clone();
        let handle_l = tokio_rt.handle().clone();
        window
            .global::<TrackInfoActions>()
            .on_load_inline(move |track_id| {
                if let Ok(id) = track_id.parse::<u64>() {
                    info_modals::load_track_info_inline(
                        runtime_l.clone(),
                        weak.clone(),
                        handle_l.clone(),
                        id,
                    );
                }
            });

        // -- Album Info --
        let weak = window.as_weak();
        window
            .global::<AlbumInfoActions>()
            .on_close(move || {
                if let Some(w) = weak.upgrade() {
                    w.global::<AlbumInfoState>().set_open(false);
                }
            });
        let weak = window.as_weak();
        window
            .global::<AlbumInfoActions>()
            .on_set_tab(move |tab| {
                if let Some(w) = weak.upgrade() {
                    w.global::<AlbumInfoState>().set_active_tab(tab);
                }
            });
        let weak = window.as_weak();
        let runtime_p = runtime.clone();
        let handle_p = tokio_rt.handle().clone();
        window
            .global::<AlbumInfoActions>()
            .on_play_track(move |id| {
                if let Some(w) = weak.upgrade() {
                    // Album view is the modal's context, so this plays the
                    // album starting at the chosen track (Tauri keeps the
                    // modal open on play).
                    playback::play_track_in_context(
                        &w,
                        runtime_p.clone(),
                        w.as_weak(),
                        handle_p.clone(),
                        &id,
                    );
                }
            });
        let weak = window.as_weak();
        let runtime_a = runtime.clone();
        let handle_a = tokio_rt.handle().clone();
        let image_cache_a = image_cache.clone();
        window
            .global::<AlbumInfoActions>()
            .on_open_label(move |label_id| {
                if let Some(w) = weak.upgrade() {
                    let name = w.global::<AlbumInfoState>().get_label().to_string();
                    w.global::<AlbumInfoState>().set_open(false);
                    if let Ok(id) = label_id.parse::<u64>() {
                        navigate_label(
                            runtime_a.clone(),
                            w.as_weak(),
                            &handle_a,
                            image_cache_a.clone(),
                            id,
                            name,
                        );
                    }
                }
            });
        let weak = window.as_weak();
        window
            .global::<AlbumInfoActions>()
            .on_open_musician(move |name, role| {
                if let Some(w) = weak.upgrade() {
                    w.global::<AlbumInfoState>().set_open(false);
                    w.global::<NetworkSidebarActions>()
                        .invoke_musician_clicked(name, role);
                }
            });
    }
}
