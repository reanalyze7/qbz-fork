//! Home/Discover/PlaylistBrowse arms of `apply_artwork`.

use slint::{ComponentHandle, Model};

use crate::artwork::target::ArtworkTarget;
use crate::{AppWindow, HomeState};

/// Handle a Home/Discover/PlaylistBrowse target. Returns `false` (unhandled)
/// for any other variant so the dispatcher can try the next category.
pub(in crate::artwork) fn apply(
    window: &AppWindow,
    target: ArtworkTarget,
    image: &slint::Image,
    pixels: &[u8],
    width: u32,
    height: u32,
) -> bool {
    let home = window.global::<HomeState>();
    match target {
        ArtworkTarget::DiscoverSectionAlbum { editor, section_idx, album_idx } => {
            let state = window.global::<crate::DiscoverState>();
            let sections = if editor {
                state.get_editor_sections()
            } else {
                state.get_home_sections()
            };
            let Some(desc) = sections.row_data(section_idx) else {
                return true;
            };
            let Some(mut item) = desc.section.albums.row_data(album_idx) else {
                return true;
            };
            item.artwork = image.clone();
            desc.section.albums.set_row_data(album_idx, item);
        }
        ArtworkTarget::Popular { idx } => {
            let popular = home.get_popular();
            if let Some(mut item) = popular.row_data(idx) {
                item.artwork = image.clone();
                popular.set_row_data(idx, item);
            }
        }
        ArtworkTarget::Recent { idx } => {
            let recent = home.get_recent();
            if let Some(mut item) = recent.row_data(idx) {
                item.artwork = image.clone();
                recent.set_row_data(idx, item);
            }
        }
        ArtworkTarget::RecentAlbum { idx } => {
            let albums = home.get_recent_albums();
            if let Some(mut item) = albums.row_data(idx) {
                item.artwork = image.clone();
                albums.set_row_data(idx, item);
            }
        }
        ArtworkTarget::MostPlayedAlbumsPage { idx } => {
            let albums = window.global::<crate::MostPlayedAlbumsState>().get_albums();
            if let Some(mut item) = albums.row_data(idx) {
                item.artwork = image.clone();
                albums.set_row_data(idx, item);
            }
        }
        ArtworkTarget::RecentAlbumsPage { idx } => {
            let albums = window.global::<crate::RecentAlbumsState>().get_albums();
            if let Some(mut item) = albums.row_data(idx) {
                item.artwork = image.clone();
                albums.set_row_data(idx, item);
            }
        }
        ArtworkTarget::HomeFavoriteAlbum { idx } => {
            let section = home.get_favorite_albums();
            if let Some(mut item) = section.albums.row_data(idx) {
                item.artwork = image.clone();
                section.albums.set_row_data(idx, item);
            }
        }
        ArtworkTarget::HomeMostPlayedAlbum { idx } => {
            let section = home.get_most_played_albums();
            if let Some(mut item) = section.albums.row_data(idx) {
                item.artwork = image.clone();
                section.albums.set_row_data(idx, item);
            }
        }
        ArtworkTarget::HomeReleaseWatchAlbum { idx } => {
            let section = home.get_release_watch();
            if let Some(mut item) = section.albums.row_data(idx) {
                item.artwork = image.clone();
                section.albums.set_row_data(idx, item);
            }
        }
        ArtworkTarget::HomeTopArtist { idx } => {
            let model = home.get_top_artists();
            if let Some(mut item) = model.row_data(idx) {
                item.artwork = image.clone();
                model.set_row_data(idx, item);
            }
        }
        ArtworkTarget::HomePlaylistCover { idx } => {
            let model = home.get_playlists();
            if let Some(mut item) = model.row_data(idx) {
                item.cover1 = image.clone(); // single cover → slot 0
                // Single-cover Discover cards letterbox a contain-fit cover with
                // its dominant colour (1:1 with Tauri's PlaylistCardLite). The
                // decoded pixels are in hand here, so compute it once on apply.
                item.dominant_color = crate::immersive::dominant_cover_color(pixels, width, height);
                model.set_row_data(idx, item);
            }
        }
        ArtworkTarget::PlaylistBrowseCover { idx } => {
            let model = window.global::<crate::PlaylistBrowseState>().get_playlists();
            if let Some(mut item) = model.row_data(idx) {
                item.cover1 = image.clone(); // single cover → slot 0
                // Same dominant-colour letterbox as HomePlaylistCover — the
                // browse grid renders the same single-cover Discover card.
                item.dominant_color = crate::immersive::dominant_cover_color(pixels, width, height);
                model.set_row_data(idx, item);
            }
        }
        ArtworkTarget::DiscoverBrowseAlbum { index } => {
            let model = window.global::<crate::DiscoverBrowseState>().get_albums();
            if let Some(mut item) = model.row_data(index) {
                item.artwork = image.clone();
                model.set_row_data(index, item);
            }
        }
        _ => return false,
    }
    true
}
