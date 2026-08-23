//! Section-model + artwork-job builders shared by `apply_home` and the
//! Slice-5 descriptor render loop.

use slint::{ModelRc, VecModel};

use crate::artwork::{ArtworkJob, ArtworkTarget};
use crate::DiscoverSection;

use super::super::{PlaylistCardData, SectionData};
use super::items::card_to_item;

/// Build the Slint section model for one tab's section set.
pub(in crate::home) fn build_sections(sections: &[SectionData]) -> Vec<DiscoverSection> {
    sections
        .iter()
        .map(|section| DiscoverSection {
            title: section.title.clone().into(),
            endpoint: section.endpoint.clone().into(),
            albums: ModelRc::new(VecModel::from(
                section.albums.iter().cloned().map(card_to_item).collect::<Vec<_>>(),
            )),
        })
        .collect()
}

/// Artwork jobs for the Qobuz Playlists row (single cover per card, so they
/// target `HomeState.playlists[idx]` directly). Skips cards with no artwork.
pub fn playlist_artwork_jobs(playlists: &[PlaylistCardData]) -> Vec<ArtworkJob> {
    playlists
        .iter()
        .enumerate()
        .filter_map(|(idx, p)| {
            (!p.artwork_url.is_empty()).then(|| ArtworkJob {
                target: ArtworkTarget::HomePlaylistCover { idx },
                url: p.artwork_url.clone(),
            })
        })
        .collect()
}
