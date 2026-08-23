//! Re-open the local album view + reset browse models after a successful save.

use slint::{ComponentHandle, ModelRc, VecModel, Weak};

use crate::AppWindow;

/// Re-open the local album view (re-splits versions with the new tags) and
/// reset the LocalLibrary browse models so the tabs re-fetch. Avoids a full
/// library reload (the 16K-track freeze).
pub(super) fn refresh_after_save(
    weak: Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    image_cache: crate::artwork::ImageCache,
) {
    let _ = weak.upgrade_in_event_loop(move |w| {
        // Refresh the open local album detail (if any) by its metadata key.
        let id = w.global::<crate::LocalAlbumState>().get_id().to_string();
        if !id.is_empty() {
            crate::local_library::open_local_album(w.as_weak(), handle.clone(), image_cache.clone(), id);
        }
        // Reset browse models so Albums/Folders/Tracks/Artists re-fetch.
        let s = w.global::<crate::LocalLibraryState>();
        let empty_albums = ModelRc::new(VecModel::from(Vec::<crate::AlbumCardItem>::new()));
        let empty_tracks = ModelRc::new(VecModel::from(Vec::<crate::TrackItem>::new()));
        s.set_albums(empty_albums.clone());
        s.set_folders(empty_albums);
        s.set_tracks(empty_tracks);
        s.set_artists(ModelRc::new(VecModel::from(Vec::<crate::LocalArtistItem>::new())));
    });
}
