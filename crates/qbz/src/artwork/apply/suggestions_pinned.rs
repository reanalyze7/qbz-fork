//! Immersive Suggestions + the mixed Pinned carousel arms of
//! `apply_artwork`. `PinnedCard`'s playlist branch needs `pixels`/
//! `width`/`height` (dominant-colour letterbox), matching `apply_artwork`'s
//! outer `pixels`/`width`/`height` params — not the local `image` alone.

use slint::{ComponentHandle, Model};

use crate::artwork::target::ArtworkTarget;
use crate::AppWindow;

pub(in crate::artwork) fn apply(
    window: &AppWindow,
    target: ArtworkTarget,
    image: &slint::Image,
    pixels: &[u8],
    width: u32,
    height: u32,
) -> bool {
    match target {
        ArtworkTarget::SuggestionCardCover { card_idx, slot } => {
            let model = window.global::<crate::SuggestionsState>().get_cards();
            if let Some(mut item) = model.row_data(card_idx) {
                match slot {
                    0 => item.cover0 = image.clone(),
                    1 => item.cover1 = image.clone(),
                    2 => item.cover2 = image.clone(),
                    3 => item.cover3 = image.clone(),
                    _ => return true,
                }
                model.set_row_data(card_idx, item);
            }
        }
        ArtworkTarget::SuggestionTrackCover { idx } => {
            let model = window.global::<crate::SuggestionsState>().get_tracks();
            if let Some(mut item) = model.row_data(idx) {
                item.artwork = image.clone();
                model.set_row_data(idx, item);
            }
        }
        ArtworkTarget::PlaylistSuggestionCover { idx } => {
            let model = window.global::<crate::PlaylistSuggestionsState>().get_rows();
            if let Some(mut item) = model.row_data(idx) {
                item.artwork = image.clone();
                model.set_row_data(idx, item);
            }
        }
        ArtworkTarget::PinnedCard { idx } => {
            let model = window.global::<crate::PinnedState>().get_items();
            if let Some(mut item) = model.row_data(idx) {
                match item.kind.as_str() {
                    "album" => item.album.artwork = image.clone(),
                    "artist" => item.artist.artwork = image.clone(),
                    "playlist" => {
                        item.playlist.cover1 = image.clone(); // single cover → slot 0
                        // Same dominant-colour letterbox as HomePlaylistCover —
                        // the pinned card renders the single-cover Discover card.
                        item.playlist.dominant_color =
                            crate::immersive::dominant_cover_color(pixels, width, height);
                    }
                    _ => return true,
                }
                model.set_row_data(idx, item);
            }
        }
        _ => return false,
    }
    true
}
