//! Slint model mappers.

use slint::ModelRc;
use slint::VecModel;

use crate::{AlbumCardItem, DiscoverSection, SlimItem};

use super::{AlbumCard, ArtistSlim};

pub(super) fn album_items(cards: &[AlbumCard]) -> Vec<AlbumCardItem> {
    cards
        .iter()
        .map(|c| AlbumCardItem {
            plays: 0,
            // Favorite heart state from the login-seeded cache (kept live by
            // main::set_album_row_favorite when a favorite toggles anywhere).
            is_favorite: crate::fav_cache::is_album_favorite(&c.id),
            // Pin badge state from the per-user pinned store (kept live by
            // main::set_album_row_pinned when a pin toggles anywhere).
            is_pinned: crate::pinned::is_pinned("album", &c.id),
            id: c.id.clone().into(),
            title: c.title.clone().into(),
            artist: c.artist.clone().into(),
            artist_id: c.artist_id.clone().into(),
            genre: "".into(),
            year: c.year.clone().into(),
            quality_tier: c.quality_tier.clone().into(),
            quality_label: c.quality_label.clone().into(),
            ribbon: "".into(),
            ribbon_kind: "".into(),
            artwork_url: c.artwork_url.clone().into(),
            artwork: slint::Image::default(),
            ..Default::default()
        })
        .collect()
}

/// `pub(crate)` since #566: `home::apply_home` reuses it for the Home
/// "Your Top Artists" rail so both tabs map artists -> items identically.
pub(crate) fn artist_items(artists: &[ArtistSlim]) -> Vec<SlimItem> {
    artists
        .iter()
        .map(|a| SlimItem {
            id: a.id.clone().into(),
            title: a.name.clone().into(),
            subtitle: "".into(),
            rank: "".into(),
            artwork_url: a.artwork_url.clone().into(),
            artwork: slint::Image::default(),
            following: a.following,
            // Pin badge state from the per-user pinned store (kept live by
            // main::set_artist_row_pinned when a pin toggles anywhere).
            is_pinned: crate::pinned::is_pinned("artist", &a.id),
        })
        .collect()
}

/// Build a `DiscoverSection` from album cards. `pub(crate)` since #566:
/// `home::apply_home` reuses it for the Home "Library Albums" rail so both
/// tabs map cards -> items identically (incl. the fav-cache heart state).
pub(crate) fn section(title: &str, cards: &[AlbumCard]) -> DiscoverSection {
    DiscoverSection {
        title: title.into(),
        // For You sections have no Discover full-list page.
        endpoint: "".into(),
        albums: ModelRc::new(VecModel::from(album_items(cards))),
    }
}
