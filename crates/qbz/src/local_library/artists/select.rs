//! Select an artist: filter their albums (in place) into the right pane.

use slint::{ComponentHandle, Model, ModelRc, VecModel};

use crate::artwork::{ArtworkJob, ArtworkTarget, ImageCache};
use crate::{AlbumCardItem, AppWindow, LocalLibraryState};

use crate::local_library::albums::map::map_local_album;

use super::matching::album_matches_artist;
use super::normalize::normalize_artist;
use super::state::ARTIST_ALBUMS;

/// Select an artist: filter their albums (in place, from the cached album
/// set) into the right pane and kick cover loads.
pub fn select_local_artist(
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    image_cache: ImageCache,
    name: String,
) {
    let _ = weak.upgrade_in_event_loop({
        let name = name.clone();
        move |w| {
            let s = w.global::<LocalLibraryState>();
            s.set_artists_selected_name(name.clone().into());
            // Display name = the merged row's display, else the raw name.
            let all = s.get_artists();
            let display = (0..all.row_count())
                .filter_map(|i| all.row_data(i))
                .find(|a| a.name == name)
                .map(|a| a.display_name.to_string())
                .unwrap_or_else(|| name.clone());
            s.set_artists_selected_display(display.into());
            s.set_artists_selected_loading(true);
        }
    });
    handle.spawn(async move {
        let cards = tokio::task::spawn_blocking(move || {
            let albums = ARTIST_ALBUMS
                .lock()
                .map(|c| c.clone())
                .unwrap_or_default();
            let nsel = normalize_artist(&name);
            albums
                .into_iter()
                .filter(|al| album_matches_artist(al, &nsel))
                .map(map_local_album)
                .collect::<Vec<_>>()
        })
        .await
        .unwrap_or_default();
        let _ = weak.upgrade_in_event_loop(move |w| {
            let jobs: Vec<ArtworkJob> = cards
                .iter()
                .enumerate()
                .filter(|(_, c)| !c.artwork_url.is_empty())
                .map(|(i, c)| ArtworkJob {
                    target: ArtworkTarget::LocalArtistAlbumCard { index: i },
                    url: c.artwork_url.clone(),
                })
                .collect();
            let items: Vec<AlbumCardItem> =
                cards.into_iter().map(crate::album_map::to_item).collect();
            let s = w.global::<LocalLibraryState>();
            s.set_artists_selected_albums(ModelRc::new(VecModel::from(items)));
            s.set_artists_selected_loading(false);
            // Source-aware: local albums carry filesystem artwork paths.
            crate::artwork::spawn_local_loads(jobs, w.as_weak(), image_cache);
        });
    });
}
