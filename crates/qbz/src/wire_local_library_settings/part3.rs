use crate::*;

pub(crate) fn wire_local_library_settings_part3(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, _settings_ctx: &Arc<settings::SettingsCtx>) {
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<TagEditorActions>()
            .on_save(move || tag_editor::save_tags(weak.clone(), handle.clone(), image_cache.clone()));
    }
    {
        let weak = window.as_weak();
        window
            .global::<TagEditorActions>()
            .on_set_persistence(move |i| {
                if let Some(w) = weak.upgrade() {
                    let s = w.global::<TagEditorState>();
                    // Ignore selecting Direct when unavailable (CUE album).
                    if i == 1 && !s.get_can_direct_write() {
                        s.set_persistence_index(0);
                    } else {
                        s.set_persistence_index(i);
                    }
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<TagEditorActions>()
            .on_set_provider(move |i| {
                if let Some(w) = weak.upgrade() {
                    w.global::<TagEditorState>().set_remote_provider_index(i);
                }
            });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<TagEditorActions>()
            .on_search_remote(move || tag_editor::search_remote(weak.clone(), handle.clone()));
    }
    {
        let weak = window.as_weak();
        window
            .global::<TagEditorActions>()
            .on_select_result(move |id| tag_editor::select_result(weak.clone(), id.to_string()));
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<TagEditorActions>()
            .on_apply_remote(move || tag_editor::apply_remote(weak.clone(), handle.clone()));
    }
    {
        let weak = window.as_weak();
        window
            .global::<TagEditorActions>()
            .on_open_in_browser(move || tag_editor::open_in_browser(weak.clone()));
    }

    // Dedicated Local album view actions (play / shuffle / edit / add / version).
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<LocalAlbumActions>().on_play_all(move || {
            if let Some(w) = weak.upgrade() {
                let tracks = local_library::current_album_version_tracks(&w);
                playback::play_local_tracks(runtime.clone(), weak.clone(), handle.clone(), tracks, 0, false);
            }
        });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<LocalAlbumActions>().on_shuffle(move || {
            if let Some(w) = weak.upgrade() {
                let tracks = local_library::current_album_version_tracks(&w);
                playback::play_local_tracks(runtime.clone(), weak.clone(), handle.clone(), tracks, 0, true);
            }
        });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<LocalAlbumActions>().on_play_track(move |id| {
            if let Some(w) = weak.upgrade() {
                let tracks = local_library::current_album_version_tracks(&w);
                let start = id
                    .parse::<i64>()
                    .ok()
                    .and_then(|tid| tracks.iter().position(|t| t.id == tid))
                    .unwrap_or(0);
                playback::play_local_tracks(runtime.clone(), weak.clone(), handle.clone(), tracks, start, false);
            }
        });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<LocalAlbumActions>().on_edit_tags(move || {
            if let Some(w) = weak.upgrade() {
                let idx = w.global::<LocalAlbumState>().get_version_index();
                if let Some(dir) = local_library::album_version_dir(idx) {
                    tag_editor::open_tag_editor(weak.clone(), handle.clone(), dir.clone(), dir);
                }
            }
        });
    }
}
