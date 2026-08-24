use crate::*;

/// Update the offline cache-status (+ progress) of every visible row matching
/// `track_id`. Mirrors `set_row_favorite`. status: 0 none / 1 queued / 2
/// downloading / 3 ready / 4 failed; `progress` is 0.0..1.0.
pub(crate) fn set_row_cache_status(window: &AppWindow, track_id: &str, status: i32, progress: f32) {
    let apply = |model: &slint::ModelRc<TrackItem>| {
        if let Some(vm) = model.as_any().downcast_ref::<slint::VecModel<TrackItem>>() {
            for i in 0..vm.row_count() {
                if let Some(mut item) = vm.row_data(i) {
                    if item.id == track_id
                        && (item.cache_status != status || item.cache_progress != progress)
                    {
                        item.cache_status = status;
                        item.cache_progress = progress;
                        vm.set_row_data(i, item);
                    }
                }
            }
        }
    };
    apply(&window.global::<AlbumState>().get_tracks());
    apply(&window.global::<ArtistState>().get_top_tracks());
    apply(&window.global::<SearchState>().get_tracks());
    apply(&window.global::<PlaylistState>().get_tracks());
    apply(&window.global::<MixState>().get_tracks());
    apply(&window.global::<FavoritesState>().get_tracks());

    let search = window.global::<SearchState>();
    let mut hero = search.get_most_popular_track();
    if hero.id == track_id {
        hero.cache_status = status;
        hero.cache_progress = progress;
        search.set_most_popular_track(hero);
    }

    // Keep the album header's "fully cached" gate live as the album's own
    // rows flip to ready (drives Make-available-offline -> Refresh in the
    // ⋯ menu). Only the open album view consults it.
    {
        let album = window.global::<AlbumState>();
        let tracks = album.get_tracks();
        let n = tracks.row_count();
        let fully = n > 0
            && (0..n).all(|i| tracks.row_data(i).is_some_and(|t| t.cache_status == 3));
        if album.get_album_fully_cached() != fully {
            album.set_album_fully_cached(fully);
        }
    }
}

/// Toggle the unlocking (padlock) flag of every visible row matching
/// `track_id`. Drives the offline-decrypt animation on the row.
pub(crate) fn set_row_unlocking(window: &AppWindow, track_id: &str, unlocking: bool) {
    let apply = |model: &slint::ModelRc<TrackItem>| {
        if let Some(vm) = model.as_any().downcast_ref::<slint::VecModel<TrackItem>>() {
            for i in 0..vm.row_count() {
                if let Some(mut item) = vm.row_data(i) {
                    if item.id == track_id && item.unlocking != unlocking {
                        item.unlocking = unlocking;
                        vm.set_row_data(i, item);
                    }
                }
            }
        }
    };
    apply(&window.global::<AlbumState>().get_tracks());
    apply(&window.global::<ArtistState>().get_top_tracks());
    apply(&window.global::<SearchState>().get_tracks());
    apply(&window.global::<PlaylistState>().get_tracks());
    apply(&window.global::<MixState>().get_tracks());
    apply(&window.global::<FavoritesState>().get_tracks());
}

/// Lazy-load the Discover > For You sections the first time the tab is
/// opened. No-op once loaded (the data persists for the session).
pub(crate) fn ensure_for_you_loaded(
    runtime: &Arc<AppRuntime<SlintAdapter>>,
    weak: &slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: &artwork::ImageCache,
) {
    let Some(w) = weak.upgrade() else {
        return;
    };
    if w.global::<ForYouState>().get_loaded() {
        return;
    }
    foryou::reset_loading(&w);
    foryou::spawn_for_you(runtime.clone(), weak.clone(), handle, image_cache.clone());
}

