//! The blocking full-load itself.

use std::sync::atomic::Ordering;

use slint::{ComponentHandle, Model, ModelRc, VecModel};

use crate::artwork::ImageCache;
use crate::{AlbumCardItem, AppWindow, LocalLibraryState};

use crate::local_library::albums::artwork::albums_dispatch_ctx;
use crate::local_library::albums::map::map_local_album;
use crate::local_library::shared::exclude_network_folders_now;

use super::state::{current_group_mode, local_albums, ALBUMS_FULL_LOAD_LIMIT, ALBUMS_GEN};

/// Full-load the metadata-grouped albums off the UI thread (mapping + cover
/// fallback resolution all happen on the blocking thread), store the raw cache
/// + the mapped card set, then derive + spawn covers on the UI thread.
pub(crate) fn spawn_albums_load(
    window: &AppWindow,
    handle: tokio::runtime::Handle,
    image_cache: ImageCache,
    gen: u64,
) {
    let weak = window.as_weak();
    let group_mode = current_group_mode(window);
    handle.spawn(async move {
        let loaded: Option<(Vec<qbz_library::LocalAlbum>, Vec<crate::album_map::AlbumCard>)> =
            tokio::task::spawn_blocking(move || {
                // include_qobuz_downloads: true (offline copies belong in the
                // grid — the toolbar's "Offline" source filter selects them).
                // exclude_network_folders: connectivity-keyed — see the
                // NETWORK-FOLDER VISIBILITY note.
                let exclude_network = exclude_network_folders_now();
                crate::library_db::with_db(|db| {
                    let page = db.get_albums_metadata_page(
                        0,
                        ALBUMS_FULL_LOAD_LIMIT,
                        None,
                        "artist",
                        "asc",
                        true,
                        exclude_network,
                        group_mode,
                    )?;
                    let albums = page.albums;
                    let cards: Vec<crate::album_map::AlbumCard> = albums
                        .iter()
                        .map(|a| {
                            let mut card = map_local_album(a.clone());
                            // Local-cover fallback scans the on-disk folder.
                            if card.artwork_url.is_empty() {
                                if let Some(cover) = db.resolve_album_cover_fallback(&card.id) {
                                    card.artwork_url = cover;
                                }
                            }
                            card
                        })
                        .collect();
                    Ok((albums, cards))
                })
            })
            .await
            .ok()
            .flatten();

        let _ = weak.upgrade_in_event_loop(move |w| {
            if ALBUMS_GEN.load(Ordering::SeqCst) != gen {
                return;
            }
            let s = w.global::<LocalLibraryState>();
            match loaded {
                Some((albums, cards)) => {
                    *local_albums() = albums;
                    let items: Vec<AlbumCardItem> =
                        cards.into_iter().map(crate::album_map::to_item).collect();
                    s.set_albums(ModelRc::new(VecModel::from(items.clone())));
                    s.set_album_count(items.len() as i32);
                    s.set_albums_loading(false);
                    s.set_albums_load_failed(false);
                    // WINDOWED artwork (was: a job for every album in the
                    // library). Stash the dispatch context BEFORE derive —
                    // derive dispatches the covers itself (flat = viewport
                    // band via dispatch_albums_window; grouped = full set).
                    *albums_dispatch_ctx().lock().unwrap() = Some(image_cache);
                    crate::local_library::albums::derive::derive_albums(&w);
                }
                None => {
                    s.set_albums_loading(false);
                    s.set_albums_load_failed(true);
                }
            }
        });
    });
}
