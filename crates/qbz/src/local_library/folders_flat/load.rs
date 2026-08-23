//! Folders tab (flat mode) load: the album grid grouped by directory rather
//! than by metadata. Full-load (folder counts are bounded; the freeze risk
//! is the Tracks table). Reuses AlbumCollectionView; covers load per card.

use slint::{ComponentHandle, Model, ModelRc, VecModel};

use crate::artwork::{ArtworkJob, ArtworkTarget, ImageCache};
use crate::{AlbumCardItem, AppWindow, LocalLibraryState};

use crate::local_library::albums::map::map_local_album;
use crate::local_library::shared::exclude_network_folders_now;

use super::derive::derive_folders;

fn fetch_folder_albums() -> Vec<crate::album_map::AlbumCard> {
    // exclude_network_folders: connectivity-keyed — an unmounted-but-online
    // folder stays visible (the index is the source); only hard offline
    // hides it (see the NETWORK-FOLDER VISIBILITY note).
    let exclude_network = exclude_network_folders_now();
    crate::library_db::with_db(move |db| {
        db.get_albums_with_full_filter(
            /* include_hidden */ false,
            /* include_qobuz_downloads */ true,
            exclude_network,
        )
    })
    .unwrap_or_default()
    .into_iter()
    .map(map_local_album)
    .collect()
}

fn folder_artwork_jobs(cards: &[crate::album_map::AlbumCard]) -> Vec<ArtworkJob> {
    cards
        .iter()
        .enumerate()
        .filter(|(_, c)| !c.artwork_url.is_empty())
        .map(|(i, c)| ArtworkJob {
            target: ArtworkTarget::LocalFolderCard { index: i },
            url: c.artwork_url.clone(),
        })
        .collect()
}

fn apply_folders(window: &AppWindow, cards: Vec<crate::album_map::AlbumCard>) {
    let items: Vec<AlbumCardItem> = cards.into_iter().map(crate::album_map::to_item).collect();
    let s = window.global::<LocalLibraryState>();
    s.set_folders(ModelRc::new(VecModel::from(items)));
    s.set_folders_loading(false);
    s.set_folders_load_failed(false);
    derive_folders(window);
}

/// Load the folder-grouped album grid on first visit (re-entry keeps it).
pub fn ensure_folders_loaded(
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    image_cache: ImageCache,
) {
    let _ = weak.upgrade_in_event_loop(move |w| {
        let s = w.global::<LocalLibraryState>();
        if s.get_folders().row_count() != 0 || s.get_folders_loading() {
            return;
        }
        s.set_folders_loading(true);
        s.set_folders_load_failed(false);
        let weak2 = w.as_weak();
        handle.spawn(async move {
            let cards = tokio::task::spawn_blocking(fetch_folder_albums)
                .await
                .unwrap_or_default();
            let _ = weak2.upgrade_in_event_loop(move |w| {
                let jobs = folder_artwork_jobs(&cards);
                apply_folders(&w, cards);
                crate::artwork::spawn_local_loads(jobs, w.as_weak(), image_cache);
            });
        });
    });
}
