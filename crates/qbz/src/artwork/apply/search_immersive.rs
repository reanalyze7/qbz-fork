//! Search / immersive-search arms of `apply_artwork` — these share the
//! URL-match late-arrival-guard idiom (a slow load from a previous query
//! must not paint the wrong cover onto a flat-index a new query reused).

use slint::{ComponentHandle, Model};

use crate::artwork::target::ArtworkTarget;
use crate::{AppWindow, SearchState};

pub(in crate::artwork) fn apply(
    window: &AppWindow,
    target: ArtworkTarget,
    _url: &str,
    image: &slint::Image,
) -> bool {
    match target {
        ArtworkTarget::SearchAlbum { idx } => {
            let model = window.global::<SearchState>().get_albums();
            if let Some(mut item) = model.row_data(idx) {
                item.artwork = image.clone();
                model.set_row_data(idx, item);
            }
        }
        ArtworkTarget::SearchTrack { idx } => {
            let model = window.global::<SearchState>().get_tracks();
            if let Some(mut item) = model.row_data(idx) {
                item.artwork = image.clone();
                model.set_row_data(idx, item);
            }
        }
        ArtworkTarget::SearchArtist { idx } => {
            let state = window.global::<SearchState>();
            let model = state.get_artists();
            let mut artist_id = None;
            if let Some(mut item) = model.row_data(idx) {
                artist_id = Some(item.id.clone());
                item.artwork = image.clone();
                model.set_row_data(idx, item);
            }
            // The All-tab carousel is a SEPARATE model (`artists_carousel`,
            // built as a clone with the hero dup dropped), so the artwork
            // pipeline never reaches it. Mirror the cover into the matching
            // carousel row by id (indices differ when the artist hero drops
            // the first entry) so the carousel cards show their images.
            if let Some(aid) = artist_id {
                let carousel = state.get_artists_carousel();
                for i in 0..carousel.row_count() {
                    if let Some(mut c) = carousel.row_data(i) {
                        if c.id == aid {
                            c.artwork = image.clone();
                            carousel.set_row_data(i, c);
                            break;
                        }
                    }
                }
            }
        }
        ArtworkTarget::SearchPlaylistCover { idx, slot } => {
            let model = window.global::<SearchState>().get_playlists();
            if let Some(mut item) = model.row_data(idx) {
                match slot {
                    0 => item.cover1 = image.clone(),
                    1 => item.cover2 = image.clone(),
                    2 => item.cover3 = image.clone(),
                    3 => item.cover4 = image.clone(),
                    _ => return true,
                }
                model.set_row_data(idx, item);
            }
        }
        ArtworkTarget::SidebarPlaylistCover { idx, slot } => {
            let model = window.global::<crate::SidebarState>().get_entries();
            if let Some(mut item) = model.row_data(idx) {
                match slot {
                    0 => item.cover1 = image.clone(),
                    1 => item.cover2 = image.clone(),
                    2 => item.cover3 = image.clone(),
                    3 => item.cover4 = image.clone(),
                    _ => return true,
                }
                model.set_row_data(idx, item);
            }
        }
        ArtworkTarget::SearchMostPopular => {
            let state = window.global::<SearchState>();
            match state.get_most_popular_kind().as_str() {
                "album" => {
                    let mut it = state.get_most_popular_album();
                    it.artwork = image.clone();
                    state.set_most_popular_album(it);
                }
                "artist" => {
                    let mut it = state.get_most_popular_artist();
                    it.artwork = image.clone();
                    state.set_most_popular_artist(it);
                }
                "track" => {
                    let mut it = state.get_most_popular_track();
                    it.artwork = image.clone();
                    state.set_most_popular_track(it);
                }
                _ => {}
            }
        }
        _ => return false,
    }
    true
}
