use crate::*;

/// Open a LOCAL album's detail in the shared AlbumPageView: load its tracks
/// (metadata-grouped), populate AlbumState with `is-local` set, then resolve
/// the folder/embedded cover from disk. `group_key` is the album's metadata
/// group key.
pub(crate) fn navigate_local_album(
    _runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: artwork::ImageCache,
    group_key: String,
) {
    let _ = weak.upgrade_in_event_loop(|w| {
        w.global::<NavState>().set_view(ContentView::LocalAlbum);
        update_nav_flags(&w);
    });
    // The dedicated local album view owns the load (versions + cover).
    local_library::open_local_album(weak, handle.clone(), image_cache, group_key);
}

/// True when an "album id" is actually a Local-Library metadata group key
/// rather than a numeric Qobuz album id. Qobuz album ids are numeric
/// strings; local group keys are `album|artist`, a folder path, or the
/// `__unknown_album__` sentinel (see qbz_library::album_grouping +
/// local_queue_track). Lets the shared `open-album` callback route local
/// items (now-playing bar, Home "Recently played", etc.) to the LocalAlbum
/// view instead of the empty Qobuz album view.
pub(crate) fn is_local_album_key(id: &str) -> bool {
    id.contains('|') || id.contains('/') || id == "__unknown_album__"
}

/// "Reveal in file explorer" (owner request, T-hires-reveal): opens the
/// track's CONTAINING FOLDER in the OS file manager. `open` has no
/// cross-desktop-portable way to select/highlight one specific file inside
/// it (that varies per file manager), so this is the same "open the folder"
/// compromise every cross-platform app without native shell-integration
/// makes.
pub(crate) fn reveal_in_file_manager(file_path: &str) {
    let Some(dir) = std::path::Path::new(file_path).parent() else {
        log::warn!("[qbz-slint] reveal-in-explorer: no parent dir for '{file_path}'");
        return;
    };
    if let Err(e) = open::that(dir) {
        log::warn!("[qbz-slint] reveal-in-explorer: failed to open '{}': {e}", dir.display());
    }
}

