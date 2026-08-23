//! Sidepanel artist-albums helpers: apply the selected artist's discography
//! sections and build their artwork jobs.

use slint::{ComponentHandle, ModelRc, VecModel};

use crate::artist::ReleaseSection;
use crate::artwork::{ArtworkJob, ArtworkTarget};
use crate::{AlbumCardItem, AppWindow, DiscoverSection, FavoritesState};

/// Apply the selected artist's albums to the sidepanel right column. Reuses
/// the standalone artist page's `/artist/page` `release_type` classifier
/// (`artist::load_artist` → `ReleaseSection`s), per the user's decision, so
/// the sections are server-authoritative (Discography / EPs & Singles / Live
/// / Compilations / Others).
pub fn apply_selected_artist(window: &AppWindow, sections: Vec<ReleaseSection>) {
    let st = window.global::<FavoritesState>();
    let mut total = 0i32;
    let ds: Vec<DiscoverSection> = sections
        .into_iter()
        .map(|s| {
            let items: Vec<AlbumCardItem> =
                s.cards.into_iter().map(crate::artist::card_to_item).collect();
            total += items.len() as i32;
            DiscoverSection {
                title: s.title.into(),
                endpoint: "".into(),
                albums: ModelRc::new(VecModel::from(items)),
            }
        })
        .collect();
    st.set_selected_artist_sections(ModelRc::new(VecModel::from(ds)));
    st.set_selected_albums_total(total);
    st.set_selected_albums_loading(false);
    st.set_selected_albums_error("".into());
}

/// Artwork jobs for the selected artist's sidepanel album cards.
pub fn selected_artist_artwork_jobs(sections: &[ReleaseSection]) -> Vec<ArtworkJob> {
    let mut jobs = Vec::new();
    for (section, sec) in sections.iter().enumerate() {
        for (index, card) in sec.cards.iter().enumerate() {
            if !card.artwork_url.is_empty() {
                jobs.push(ArtworkJob {
                    url: card.artwork_url.clone(),
                    target: ArtworkTarget::FavoriteArtistAlbum { section, index },
                });
            }
        }
    }
    jobs
}
