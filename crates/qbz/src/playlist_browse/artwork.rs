//! Tiny pure helper building `ArtworkJob`s from a page of cards.

use super::model::BrowseCard;
use crate::artwork::{ArtworkJob, ArtworkTarget};

/// Artwork jobs for a page of cards, targeting their absolute indices in
/// `PlaylistBrowseState.playlists` (`base_index` is the model length
/// before the page was appended).
pub(super) fn artwork_jobs(cards: &[BrowseCard], base_index: usize) -> Vec<ArtworkJob> {
    cards
        .iter()
        .enumerate()
        .filter(|(_, bc)| !bc.card.artwork_url.is_empty())
        .map(|(i, bc)| ArtworkJob {
            url: bc.card.artwork_url.clone(),
            target: ArtworkTarget::PlaylistBrowseCover {
                idx: base_index + i,
            },
        })
        .collect()
}
